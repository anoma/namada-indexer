use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum_macros::debug_handler;

use crate::error::api::ApiError;
use crate::response::revealed_pk::RevealedPk;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_revealed_pk(
    _headers: HeaderMap,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<RevealedPk>, ApiError> {
    let revealed_pk = state
        .revealed_pk_service
        .get_revealed_pk_by_address(&state.client, address)
        .await?;

    Ok(Json(revealed_pk))
}
