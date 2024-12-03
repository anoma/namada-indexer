use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum IbcError {
    #[error("Revealed public key {0} not found")]
    NotFound(u64),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for IbcError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            IbcError::NotFound(_) => StatusCode::NOT_FOUND,
            IbcError::Unknown(_) | IbcError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
