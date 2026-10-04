use std::marker::PhantomData;

use anyhow::Context;
use axum::{body::Bytes, extract::ws::{Message, Utf8Bytes}};
use futures_util::{Sink, SinkExt, Stream, StreamExt};

use crate::api::ws::error::{WsHandlerError, WsHandlerResult};

pub struct WebSocketJsonSink<T, Inner>
where
    T: serde::Serialize,
    Inner: Sink<Message, Error: Into<anyhow::Error>> + Unpin,
{
    inner: Inner,
    _type_phantom: PhantomData<T>,
}

pub struct WebSocketJsonStream<T, Inner>
where
    T: serde::Deserialize<'static>,
    Inner: Stream<Item = Result<Message, axum::Error>> + Unpin,
{
    inner: Inner,
    _type_phantom: PhantomData<T>,
}

impl<T, Inner> WebSocketJsonSink<T, Inner>
where
    T: serde::Serialize,
    Inner: Sink<Message, Error: Into<anyhow::Error>> + Unpin,
{
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            _type_phantom: PhantomData,
        }
    }

    pub async fn send(&mut self, data: T) -> WsHandlerResult<()> {
        let serialized_data = serde_json::to_string(&data)
            .context("Serializing data as JSON for WebSocket stream")
            .map_err(WsHandlerError::internal)?;

        let serialized_data_utf_bytes: Utf8Bytes = serialized_data.into();
        self.send_raw(Message::Text(serialized_data_utf_bytes)).await
    }

    pub async fn send_raw(&mut self, message: Message) -> WsHandlerResult<()> {
        self.inner.send(message)
            .await
            .map_err(WsHandlerError::connection)
    }
}

impl<T, Inner> WebSocketJsonStream<T, Inner>
where
    T: serde::de::DeserializeOwned + Send + 'static,
    Inner: Stream<Item = Result<Message, axum::Error>> + Unpin,
{
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            _type_phantom: PhantomData,
        }
    }

    pub async fn recv(&mut self) -> WsHandlerResult<Option<T>> {
        let received_data_bytes: Bytes = loop {
            let rx_result = self.inner.next().await
                .transpose()
                .context("Receiving message from WebSocket stream")
                .map_err(WsHandlerError::connection)?;

            match rx_result {
                // Websocket closed abruptly
                None => return Ok(None),
                // Websocket closing gracefully
                Some(Message::Close(..)) => return Ok(None),
                // Irrelevant control frames
                Some(Message::Ping(..) | Message::Pong(..)) => continue,
                // Actual data
                Some(Message::Binary(data_bytes)) => break data_bytes,
                Some(Message::Text(data_utf_bytes)) => break data_utf_bytes.into(),
            }
        };

        let deserialized_data_result = tokio::task::spawn_blocking(move || serde_json::from_slice::<T>(&received_data_bytes))
            .await
            .context("Deserializing JSON data from WebSocket client")
            .map_err(WsHandlerError::internal)?;

        let deserialized_data = deserialized_data_result
            .context("Deserializing JSON data")
            .map_err(WsHandlerError::protocol)?;

        Ok(Some(deserialized_data))
    }
}
