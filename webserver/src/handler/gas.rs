use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;

use crate::error::api::ApiError;
use crate::response::gas::{Gas, GasPrice};
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
