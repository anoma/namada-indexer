use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

use crate::error::api::ApiError;
use crate::response::pos::ValidatorResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_validators(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    State(state): State<CommonState>,
) -> Result<Json<Vec<ValidatorResponse>>, ApiError> {
    let validators = state.pos_service.get_all_validators().await;

    let response = validators
        .into_iter()
        .map(|v| ValidatorResponse {
            address: v.address.to_string(),
            voting_power: v.voting_power,
            max_commission: v.max_commission,
            commission: v.commission,
            email: v.email,
            description: v.description,
            website: v.website,
            discord_handle: v.discord_handler,
            avatar: v.avatar,
        })
        .collect();

    return Ok(Json(response));
}
