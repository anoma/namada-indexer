use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum BlockError {
    #[error("Block not found error at {0}: {1}")]
    NotFound(String, String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for BlockError {
    fn into_response(self) -> Response {
        let status_code = match self {
            BlockError::Unknown(_) | BlockError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            BlockError::NotFound(_, _) => StatusCode::NOT_FOUND,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
