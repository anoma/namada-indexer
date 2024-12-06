use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;

use crate::error::api::ApiError;
use crate::response::block::Block;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_block_by_height(
    _headers: HeaderMap,
    Path(height): Path<i32>,
    State(state): State<CommonState>,
) -> Result<Json<Block>, ApiError> {
    let block = state.block_service.get_block_by_height(height).await?;

    Ok(Json(block))
}

#[debug_handler]
pub async fn get_block_by_timestamp(
    _headers: HeaderMap,
    Path(timestamp): Path<i64>,
    State(state): State<CommonState>,
) -> Result<Json<Block>, ApiError> {
    let block = state
        .block_service
        .get_block_by_timestamp(timestamp)
        .await?;

    Ok(Json(block))
}
