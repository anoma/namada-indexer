use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("The tx id must be 32bytes long")]
    InvalidTxId,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Rpc error: {0}")]
    Rpc(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for TransactionError {
    fn into_response(self) -> Response {
        let status_code = match self {
            TransactionError::InvalidTxId => StatusCode::BAD_REQUEST,
            TransactionError::Unknown(_)
            | TransactionError::Database(_)
            | TransactionError::Rpc(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
