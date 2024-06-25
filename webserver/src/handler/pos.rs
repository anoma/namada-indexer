use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_extra::extract::Query;
use axum_macros::debug_handler;

use crate::dto::pos::{
    BondsDto, MyValidatorQueryParams, PoSQueryParams, UnbondsDto,
    ValidatorStateDto, WithdrawsDto,
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
    Query(query): Query<PoSQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<ValidatorWithId>>>, ApiError> {
    let page = query.page.unwrap_or(1);
    let states = query.state.unwrap_or_else(ValidatorStateDto::all);
    let (validators, total_validators) =
        state.pos_service.get_all_validators(page, states).await?;

    let response = PaginatedResponse::new(validators, page, total_validators);
    Ok(Json(response))
}

pub async fn get_my_validators(
    _headers: HeaderMap,
    query: Query<MyValidatorQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<ValidatorWithId>>>, ApiError> {
    // TODO: validate that query.address contains valid bech32m  encoded
    // addresses
    let page = query.page.unwrap_or(1);
    let (validators, total_validators) = state
        .pos_service
        .get_my_validators(page, query.addresses.clone(), query.kind.clone())
        .await?;

    let response = PaginatedResponse::new(validators, page, total_validators);
    Ok(Json(response))
}

#[debug_handler]
pub async fn get_bonds(
    _headers: HeaderMap,
    query: Query<BondsDto>,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Bond>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (bonds, total_bonds) = state
        .pos_service
        .get_bonds_by_address(address, page)
        .await?;

    let response = PaginatedResponse::new(bonds, page, total_bonds);

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

    let (bonds, total_bonds) = state
        .pos_service
        .get_merged_bonds_by_address(address, page)
        .await?;

    let response = PaginatedResponse::new(bonds, page, total_bonds);

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

    let (unbonds, total_unbonds) = state
        .pos_service
        .get_unbonds_by_address(address, page)
        .await?;

    let response = PaginatedResponse::new(unbonds, page, total_unbonds);

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

    let (unbonds, total_unbonds) = state
        .pos_service
        .get_merged_unbonds_by_address(address, page)
        .await?;

    let response = PaginatedResponse::new(unbonds, page, total_unbonds);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_withdraws(
    _headers: HeaderMap,
    query: Query<WithdrawsDto>,
    Path((address, epoch)): Path<(String, u64)>,
    State(state): State<CommonState>,
) -> Result<Json<PaginatedResponse<Vec<Withdraw>>>, ApiError> {
    let page = query.page.unwrap_or(1);

    let (withdraws, total_withdraws) = state
        .pos_service
        .get_withdraws_by_address(address, epoch, page)
        .await?;

    let response = PaginatedResponse::new(withdraws, page, total_withdraws);

    Ok(Json(response))
}

#[debug_handler]
pub async fn get_rewards(
    _headers: HeaderMap,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Reward>>, ApiError> {
    let rewards = state.pos_service.get_rewards_by_address(address).await?;
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
