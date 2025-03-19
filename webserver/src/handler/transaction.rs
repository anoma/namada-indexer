use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum_extra::extract::Query;
use axum_macros::debug_handler;
use subtle_encoding::hex;

use crate::dto::transaction::TransactionHistoryQueryParams;
use crate::error::api::ApiError;
use crate::error::transaction::TransactionError;
use crate::response::transaction::{
    InnerTransactionResponse, TransactionHistoryResponse,
    WrapperTransactionResponse,
};
use crate::response::utils::PaginatedResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_wrapper_tx(
    _headers: HeaderMap,
    Path(tx_id): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Option<(WrapperTransactionResponse)>>, ApiError> {
    is_valid_hash(&tx_id)?;
    let tx_id = tx_id.to_lowercase();

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

    let response = wrapper_tx
        .map(|wrapper| WrapperTransactionResponse::new(wrapper, inner_txs));

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_inner_tx(
    _headers: HeaderMap,
    Path(tx_id): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Option<InnerTransactionResponse>>, ApiError> {
    is_valid_hash(&tx_id)?;
    let tx_id = tx_id.to_lowercase();

    let inner_tx = state
        .transaction_service
        .get_inner_tx(tx_id.clone())
        .await?;

    if inner_tx.is_none() {
        return Err(TransactionError::TxIdNotFound(tx_id).into());
    }

    let response = inner_tx.map(|inner| InnerTransactionResponse::new(inner));

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_transaction_history(
    _headers: HeaderMap,
    Query(query): Query<TransactionHistoryQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<TransactionHistoryResponse>>>, ApiError>
{
    let page = query.page.unwrap_or(1);

    let (transactions, total_pages, total_items) = state
        .transaction_service
        .get_addresses_history(query.addresses, page)
        .await?;

    let response = transactions
        .into_iter()
        .map(TransactionHistoryResponse::from)
        .collect();

    Ok(Json(PaginatedResponse::new(
        response,
        page,
        total_pages,
        total_items,
    )))
}

fn is_valid_hash(hash: &str) -> Result<(), TransactionError> {
    let is_valid_lenght = hash.len().eq(&64);
    let is_valid_hex = hex::decode(hash.as_bytes()).is_ok();
    if is_valid_lenght && is_valid_hex {
        Ok(())
    } else {
        Err(TransactionError::InvalidTxId)
    }
}
