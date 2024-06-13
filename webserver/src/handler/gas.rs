use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;

use crate::error::api::ApiError;
use crate::response::gas::Gas;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_gas_by_token(
    _headers: HeaderMap,
    Path(token): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Gas>>, ApiError> {
    let gas = state.gas_service.get_gas_by_token(token).await;

    Ok(Json(gas))
}
