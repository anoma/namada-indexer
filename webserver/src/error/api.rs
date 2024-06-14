use axum::response::{IntoResponse, Response};
use thiserror::Error;

use super::balance::BalanceError;
use super::chain::ChainError;
use super::gas::GasError;
use super::governance::GovernanceError;
use super::pos::PoSError;
use super::revealed_pk::RevealedPkError;
use super::transaction::TransactionError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error(transparent)]
    ChainError(#[from] ChainError),
    #[error(transparent)]
    PoSError(#[from] PoSError),
    #[error(transparent)]
    BalanceError(#[from] BalanceError),
    #[error(transparent)]
    GovernanceError(#[from] GovernanceError),
    #[error(transparent)]
    RevealedPkError(#[from] RevealedPkError),
    #[error(transparent)]
    GasError(#[from] GasError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::TransactionError(error) => error.into_response(),
            ApiError::ChainError(error) => error.into_response(),
            ApiError::PoSError(error) => error.into_response(),
            ApiError::BalanceError(error) => error.into_response(),
            ApiError::GovernanceError(error) => error.into_response(),
            ApiError::RevealedPkError(error) => error.into_response(),
            ApiError::GasError(error) => error.into_response(),
        }
    }
}
