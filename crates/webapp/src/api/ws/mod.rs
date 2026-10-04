use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{StreamExt, stream::{SplitSink, SplitStream}};
use liblife::{LifeView, LifeViewMut, chunked::LifeChunkTilePos};
use tracing::{debug, error};

use crate::api::{state::board::{ApiStateBoardHandle, ApiStateBoardId}, types::{self, WsClientbound, WsClientboundChunk, WsServerbound, WsServerboundPullChunk, WsServerboundPushChunkTile, WsServerboundSetWatchedChunks}, ws::{error::{WsHandlerError, WsHandlerResult}, json::{WebSocketJsonSink, WebSocketJsonStream}}};

mod error;
mod json;

pub struct BoardWebSocketHandler {
    client_state: BoardWebSocketClientState,
    tx: WebSocketJsonSink<WsClientbound, SplitSink<WebSocket, Message>>,
    rx: WebSocketJsonStream<WsServerbound, SplitStream<WebSocket>>,
    board_id: ApiStateBoardId,
    board_handle: ApiStateBoardHandle,
}

#[derive(Debug, Clone, Default)]
struct BoardWebSocketClientState {
    pub watched_chunk_positions: Vec<LifeChunkTilePos>,
}

impl BoardWebSocketHandler {
    pub fn new(ws: WebSocket, board_id: ApiStateBoardId, board_handle: ApiStateBoardHandle) -> Self {
        let (tx_raw, rx_raw) = ws.split();
        let tx = WebSocketJsonSink::new(tx_raw);
        let rx = WebSocketJsonStream::new(rx_raw);

        Self {
            client_state: BoardWebSocketClientState::default(),
            tx,
            rx,
            board_id,
            board_handle,
        }
    }

    pub async fn handle(mut self) {
        match self.handle_comm_loop().await {
            Ok(_) => {
                // Client initiated a graceful closing and has nothing more to say,
                // respond with a close message to acknowledge the graceful closing.
                let _ = self.tx.send_raw(Message::Close(None));
            },
            Err(WsHandlerError::Connection(e)) => {
                // Error communicating with the client, don't bother trying to communicate again.
                debug!(
                    error = %e,
                    backtrace = %e.backtrace(),
                    "Error communicating with board websocket client; Dropping client.",
                );
            },
            Err(WsHandlerError::Internal(e)) => {
                // We fucked something up, log the error and drop the client with a generalized explanation.
                error!(
                    error = %e,
                    backtrace = %e.backtrace(),
                    "Internal error handling board websocket client",
                );
                let _ = self.tx.send_raw(Message::Close(Some(CloseFrame {
                    code: 500,
                    reason: "Internal server error; Closing abruptly.".into(),
                })));
            },
            Err(WsHandlerError::Protocol(e)) => {
                // The client fucked something up, respond by telling what they did wrong and drop the client.
                debug!(
                    error = %e,
                    backtrace = %e.backtrace(),
                    "board websocket client misbehaved; Dropping client.",
                );
                let _ = self.tx.send_raw(Message::Close(Some(CloseFrame {
                    code: 400,
                    reason: format!("Client misbehavior: {e}").into(),
                })));
            }
        }
    }

    async fn handle_comm_loop(&mut self) -> WsHandlerResult<()> {
        {
            let ping_frame_bytes = env!("CARGO_PKG_VERSION").into();
            self.tx.send_raw(Message::Ping(ping_frame_bytes)).await?;
        }

        loop {
            let msg = match self.rx.recv().await? {
                Some(msg) => msg,
                None => break,
            };

            match msg {
                WsServerbound::SetWatchedChunks(WsServerboundSetWatchedChunks { watched_chunk_positions }) => {
                    self.client_state.watched_chunk_positions = watched_chunk_positions.into_iter()
                        .map(LifeChunkTilePos::from)
                        .collect();
                },
                WsServerbound::PullChunk(WsServerboundPullChunk { chunk_pos }) => {
                    let (chunk_width, chunk_tile_states) = {
                        let board = self.board_handle.board.read().await;

                        let board_chunk = board
                            .chunked(types::SERIAL_BOARD_CHUNK_LENGTH, types::SERIAL_BOARD_CHUNK_LENGTH)
                            .at_checked(chunk_pos.into())
                            .ok_or(WsHandlerError::protocol_from_msg("Pulled chunk position is out of bounds"))?;

                        (
                            board_chunk.width(),
                            board_chunk.iter_tiles().collect()
                        )
                    };

                    self.tx.send(WsClientbound::Chunk(WsClientboundChunk {
                        chunk_pos,
                        chunk_width,
                        chunk_tile_states,
                    })).await?;
                },
                WsServerbound::PushChunkTile(WsServerboundPushChunkTile { chunk_pos, chunk_tile_pos, tile_state }) => {
                    let mut board = self.board_handle.board.write().await;

                    let mut board_chunk = board
                        .chunked_mut(types::SERIAL_BOARD_CHUNK_LENGTH, types::SERIAL_BOARD_CHUNK_LENGTH)
                        .take_at_mut_checked(chunk_pos.into())
                        .ok_or(WsHandlerError::protocol_from_msg("Pushed chunk position is out of bounds"))?;

                    board_chunk.set_tile_checked(chunk_tile_pos.into(), tile_state)
                        .ok_or(WsHandlerError::protocol_from_msg("Pushed chunk tile position is out of bounds"))?;
                },
            }
        }

        Ok(())
    }
}
