use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

use crate::error::api::ApiError;
use crate::response::pos::{Bond, Reward, Unbond, ValidatorWithId, Withdraw};
use crate::response::utils::PaginatedResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_validators(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Query(page): Query<u64>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<ValidatorWithId>>>, ApiError> {
    let (validators, total_validators) =
        state.pos_service.get_all_validators(page).await?;

    let response = PaginatedResponse::new(validators, page, total_validators);
    Ok(Json(response))
}

#[debug_handler]
pub async fn get_bonds(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Bond>>, ApiError> {
    let bonds = state.pos_service.get_bonds_by_address(address).await?;
    Ok(Json(bonds))
}

#[debug_handler]
pub async fn get_unbonds(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Unbond>>, ApiError> {
    let bonds = state.pos_service.get_unbonds_by_address(address).await?;
    Ok(Json(bonds))
}

#[debug_handler]
pub async fn get_withdraws(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Path((address, epoch)): Path<(String, u64)>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Withdraw>>, ApiError> {
    let withdraws = state
        .pos_service
        .get_withdraws_by_address(address, epoch)
        .await?;
    Ok(Json(withdraws))
}

#[debug_handler]
pub async fn get_rewards(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Reward>>, ApiError> {
    let rewards = state.pos_service.get_rewards_by_address(address).await?;
    Ok(Json(rewards))
}
