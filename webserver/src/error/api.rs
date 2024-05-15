use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use super::balance::BalanceError;
use super::governance::GovernanceError;
use crate::response::api::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    BalanceError(#[from] BalanceError),
    #[error(transparent)]
    GovernanceError(#[from] GovernanceError),
    #[error("No chain parameters stored")]
    NoChainParameters,
    #[error("Invalid request header")]
    InvalidHeader,
    #[error("Invalid form signature")]
    InvalidFormSignature,
    #[error("Failed form submission")]
    FailedFormSubmission,
    #[error("Failed saving the special task")]
    NetworkError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BalanceError(error) => error.into_response(),
            ApiError::GovernanceError(error) => error.into_response(),
            ApiError::InvalidHeader => ApiErrorResponse::send(
                StatusCode::BAD_REQUEST.as_u16(),
                Some("Invalid Header".to_string()),
            ),
            ApiError::NoChainParameters => ApiErrorResponse::send(
                StatusCode::NOT_FOUND.as_u16(),
                Some("Chain parameters not found".to_string()),
            ),
            ApiError::InvalidFormSignature => ApiErrorResponse::send(
                StatusCode::BAD_REQUEST.as_u16(),
                Some("Invalid form signature".to_string()),
            ),
            ApiError::FailedFormSubmission => ApiErrorResponse::send(
                StatusCode::FORBIDDEN.as_u16(),
                Some(
                    "Player is not part of the shielded expedition".to_string(),
                ),
            ),
            ApiError::NetworkError => ApiErrorResponse::send(
                StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                Some("Failed saving the task. Please retry.".to_string()),
            ),
        }
    }
}
