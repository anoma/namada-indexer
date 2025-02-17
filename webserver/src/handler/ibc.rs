use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum_extra::extract::Query;
use axum_macros::debug_handler;
use bigdecimal::BigDecimal;

use crate::dto::ibc::IbcRateLimit as IbcRateLimitDto;
use crate::error::api::ApiError;
use crate::response::ibc::{IbcAck, IbcRateLimit};
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

#[debug_handler]
pub async fn get_ibc_rate_limits(
    Query(query): Query<IbcRateLimitDto>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<IbcRateLimit>>, ApiError> {
    let rate_limits = state
        .ibc_service
        .get_throughput_limits(
            query.token_address,
            query.throughput_limit.map(BigDecimal::from),
        )
        .await?;

    Ok(Json(rate_limits))
}
