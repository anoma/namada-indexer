use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum GasError {
    #[error("Invalid query parameters")]
    InvalidQueryParams,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for GasError {
    fn into_response(self) -> Response {
        let status_code = match self {
            GasError::InvalidQueryParams => StatusCode::BAD_GATEWAY,
            GasError::Unknown(_) | GasError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
