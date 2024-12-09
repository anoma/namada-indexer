use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Too Short pattern, minimum character 3, got {0}")]
    TooShortPattern(usize),
    #[error("Proposal {0} not found")]
    NotFound(u64),
    #[error("Proposal {0} has no associated data")]
    DataNotFound(u64),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl IntoResponse for GovernanceError {
    fn into_response(self) -> Response {
        let status_code = match self {
            GovernanceError::TooShortPattern(_) => StatusCode::BAD_REQUEST,
            GovernanceError::NotFound(_) => StatusCode::NOT_FOUND,
            GovernanceError::DataNotFound(_) => StatusCode::NOT_FOUND,
            GovernanceError::Unknown(_) | GovernanceError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
