use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;

use crate::error::api::ApiError;
use crate::response::ibc::IbcAck;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_ibc_status(
    _headers: HeaderMap,
    Path(tx_id): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<IbcAck>, ApiError> {
    let ibc_ack_status = state.ibc_service.get_ack_by_tx_id(tx_id).await?;

    Ok(Json(ibc_ack_status))
}
