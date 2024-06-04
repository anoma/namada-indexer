use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

use crate::error::api::ApiError;
use crate::response::gas::GasCost;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_gas_table(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    State(state): State<CommonState>,
) -> Result<Json<Vec<GasCost>>, ApiError> {
    let gas_table = state.gas_service.get_gas_table(&state.client).await;

    Ok(Json(gas_table))
}
