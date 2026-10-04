use anyhow::anyhow;

pub type WsHandlerResult<T> = Result<T, WsHandlerError>;

#[derive(Debug)]
pub enum WsHandlerError {
    /// Error caused by server misbehavior
    Internal(anyhow::Error),
    /// Error caused by a broken connection
    Connection(anyhow::Error),
    /// Error caused by client misbehavior
    Protocol(anyhow::Error),
}

impl WsHandlerError {
    pub fn internal(err: impl Into<anyhow::Error>) -> Self {
        Self::Internal(err.into())
    }

    pub fn connection(err: impl Into<anyhow::Error>) -> Self {
        Self::Connection(err.into())
    }

    pub fn protocol(err: impl Into<anyhow::Error>) -> Self {
        Self::Protocol(err.into())
    }

    pub fn internal_from_msg(msg: impl ToString) -> Self {
        Self::internal(anyhow!(msg.to_string()))
    }

    pub fn connection_from_msg(msg: impl ToString) -> Self {
        Self::connection(anyhow!(msg.to_string()))
    }

    pub fn protocol_from_msg(msg: impl ToString) -> Self {
        Self::protocol(anyhow!(msg.to_string()))
    }
}
