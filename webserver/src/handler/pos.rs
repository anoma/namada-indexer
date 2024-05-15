use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

use crate::error::api::ApiError;
use crate::response::pos::Validator;
use crate::response::utils::PaginatedResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_validators(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Query(page): Query<u64>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Validator>>>, ApiError> {
    let (validators, total_validators) =
        state.pos_service.get_all_validators(page).await?;

    let response = PaginatedResponse::new(validators, page, total_validators);
    Ok(Json(response))
}
