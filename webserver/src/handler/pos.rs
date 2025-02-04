use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_extra::extract::Query;
use axum_macros::debug_handler;

use crate::dto::pos::{
    AllValidatorsQueryParams, BondsDto, RewardsDto, UnbondsDto,
    ValidatorQueryParams, ValidatorStateDto, WithdrawsDto,
};
use crate::error::api::ApiError;
use crate::response::pos::{
    Bond, MergedBond, Reward, TotalVotingPower, Unbond, ValidatorWithId,
    Withdraw,
};
use crate::response::utils::PaginatedResponse;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_validators(
    _headers: HeaderMap,
    Query(query): Query<ValidatorQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<ValidatorWithId>>>, ApiError> {
    let page = query.page.unwrap_or(1);
    let states = query.state.unwrap_or_else(ValidatorStateDto::all);
    let (validators, total_pages, total_validators) = state
        .pos_service
        .get_validators(page, states, query.sort_field, query.sort_order)
        .await?;

    let response =
        PaginatedResponse::new(validators, page, total_pages, total_validators);
    Ok(Json(response))
}

#[debug_handler]
pub async fn get_all_validators(
    _headers: HeaderMap,
    Query(query): Query<AllValidatorsQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<ValidatorWithId>>, ApiError> {
    let states = query.state.unwrap_or_else(ValidatorStateDto::all);
    let validators = state.pos_service.get_all_validators(states).await?;

    Ok(Json(validators))
}

#[debug_handler]
pub async fn get_bonds(
    _headers: HeaderMap,
    query: Query<BondsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Bond>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (bonds, total_pages, total_bonds) = state
        .pos_service
        .get_bonds_by_address(address, page)
        .await?;

    let response =
        PaginatedResponse::new(bonds, page, total_pages, total_bonds);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_merged_bonds(
    _headers: HeaderMap,
    query: Query<BondsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<MergedBond>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (bonds, total_pages, total_bonds) = state
        .pos_service
        .get_merged_bonds_by_address(address, page)
        .await?;

    let response =
        PaginatedResponse::new(bonds, page, total_pages, total_bonds);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_unbonds(
    _headers: HeaderMap,
    query: Query<UnbondsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Unbond>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (unbonds, total_pages, total_unbonds) = state
        .pos_service
        .get_unbonds_by_address(address, page)
        .await?;

    let response =
        PaginatedResponse::new(unbonds, page, total_pages, total_unbonds);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_merged_unbonds(
    _headers: HeaderMap,
    query: Query<UnbondsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Unbond>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (unbonds, total_pages, total_unbonds) = state
        .pos_service
        .get_merged_unbonds_by_address(address, page)
        .await?;

    let response =
        PaginatedResponse::new(unbonds, page, total_pages, total_unbonds);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_withdraws(
    _headers: HeaderMap,
    query: Query<WithdrawsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Withdraw>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (withdraws, total_pages, total_withdraws) = state
        .pos_service
        .get_withdraws_by_address(address, query.epoch, page)
        .await?;

    let response =
        PaginatedResponse::new(withdraws, page, total_pages, total_withdraws);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_rewards(
    _headers: HeaderMap,
    query: Query<RewardsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Reward>>, ApiError> {
    let rewards = state
        .pos_service
        .get_rewards_by_address(address, query.epoch)
        .await?;
    Ok(Json(rewards))
}

#[debug_handler]
pub async fn get_rewards_by_delegator_and_validator_and_epoch(
    _headers: HeaderMap,
    Path((delegator, validator, epoch)): Path<(String, String, u64)>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Reward>>, ApiError> {
    let rewards = state
        .pos_service
        .get_rewards_by_delegator_and_validator_and_epoch(
            delegator, validator, epoch,
        )
        .await?;
    Ok(Json(rewards))
}

#[debug_handler]
pub async fn get_total_voting_power(
    _headers: HeaderMap,
    State(state): State<CommonState>,
) -> Result<Json<TotalVotingPower>, ApiError> {
    let total_voting_power = state.pos_service.get_total_voting_power().await?;
    Ok(Json(TotalVotingPower {
        total_voting_power: total_voting_power.to_string(),
    }))
}
