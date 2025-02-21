use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum_macros::debug_handler;

use crate::dto::gas::GasEstimateQuery;
use crate::error::api::ApiError;
use crate::response::gas::{Gas, GasEstimate, GasPrice};
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_gas(
    _headers: HeaderMap,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Gas>>, ApiError> {
    let gas = state.gas_service.get_gas().await;

    Ok(Json(gas))
}

#[debug_handler]
pub async fn get_gas_price_by_token(
    _headers: HeaderMap,
    Path(token): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<GasPrice>>, ApiError> {
    let gas_price = state.gas_service.get_gas_price_by_token(token).await?;

    Ok(Json(gas_price))
}

#[debug_handler]
pub async fn get_all_gas_prices(
    _headers: HeaderMap,
    State(state): State<CommonState>,
) -> Result<Json<Vec<GasPrice>>, ApiError> {
    let gas_price = state.gas_service.get_all_gas_prices().await?;

    Ok(Json(gas_price))
}

#[debug_handler]
pub async fn get_gas_estimate(
    _headers: HeaderMap,
    Query(query): Query<GasEstimateQuery>,
    State(state): State<CommonState>,
) -> Result<Json<GasEstimate>, ApiError> {
    query.is_valid()?;

    let gas = state
        .gas_service
        .estimate_gas(
            query.bond.unwrap_or(0),
            query.redelegate.unwrap_or(0),
            query.claim_rewards.unwrap_or(0),
            query.unbond.unwrap_or(0),
            query.transparent_transfer.unwrap_or(0),
            query.shielded_transfer.unwrap_or(0),
            query.shielding_transfer.unwrap_or(0),
            query.unshielding_transfer.unwrap_or(0),
            query.vote.unwrap_or(0),
            query.ibc.unwrap_or(0),
            query.withdraw.unwrap_or(0),
            query.reveal_pk.unwrap_or(0),
            query.signatures.unwrap_or(2),
            query.tx_size.unwrap_or(0),
        )
        .await?;

    Ok(Json(gas))
}
