// Mostly taken from https://github.com/tokio-rs/axum/blob/3f8956dcd007070be5449dd23102b7ee7f2e1b05/examples/anyhow-error-response/src/main.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type ApiResult<T> = Result<T, ApiError>;

// Make our own error that wraps `anyhow::Error`.
pub struct ApiError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
