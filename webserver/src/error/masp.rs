use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

use crate::response::api::ApiErrorResponse;
#[derive(Error, Debug)]
pub enum MaspError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for MaspError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            MaspError::Unknown(_) | MaspError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
