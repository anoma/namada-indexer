use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;

use crate::error::api::ApiError;
use crate::error::transaction::TransactionError;
use crate::response::transaction::{InnerTransaction, WrapperTransaction};
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_wrapper_tx(
    _headers: HeaderMap,
    Path(tx_id): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Option<WrapperTransaction>>, ApiError> {
    is_valid_hash(&tx_id)?;

    let wrapper_tx = state
        .transaction_service
        .get_wrapper_tx(tx_id.clone())
        .await?;

    if wrapper_tx.is_none() {
        return Err(TransactionError::TxIdNotFound(tx_id).into());
    }

    let inner_txs = state
        .transaction_service
        .get_inner_tx_by_wrapper_id(tx_id)
        .await?;

    Ok(Json(wrapper_tx.map(|mut wrapper| {
        wrapper.inner_transactions =
            inner_txs.into_iter().map(|tx| tx.to_short()).collect();
        wrapper
    })))
}

#[debug_handler]
pub async fn get_inner_tx(
    _headers: HeaderMap,
    Path(tx_id): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Option<InnerTransaction>>, ApiError> {
    is_valid_hash(&tx_id)?;

    let inner_tx = state
        .transaction_service
        .get_inner_tx(tx_id.clone())
        .await?;

    if inner_tx.is_none() {
        return Err(TransactionError::TxIdNotFound(tx_id).into());
    }

    Ok(Json(inner_tx))
}

fn is_valid_hash(hash: &str) -> Result<(), TransactionError> {
    if hash.len().eq(&64) {
        Ok(())
    } else {
        Err(TransactionError::InvalidTxId)
    }
}
