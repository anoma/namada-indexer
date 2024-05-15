use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;
use axum_trace_id::TraceId;

use crate::dto::governance::ProposalQueryParams;
use crate::error::api::ApiError;
use crate::error::governance::GovernanceError;
use crate::response::governance::Proposal;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_governance_proposals(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Query(query): Query<ProposalQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<Proposal>>, ApiError> {
    let proposals = state
        .gov_service
        .find_governance_proposals(query.status, query.pagination)
        .await?;

    Ok(Json(proposals))
}

#[debug_handler]
pub async fn get_governance_proposal_by_id(
    _trace_id: TraceId<String>,
    _headers: HeaderMap,
    Path(proposal_id): Path<u64>,
    State(state): State<CommonState>,
) -> Result<Json<Proposal>, ApiError> {
    let proposal = state
        .gov_service
        .find_governance_proposal_by_id(proposal_id)
        .await?;

    if let Some(proposal) = proposal {
        Ok(Json(proposal))
    } else {
        Err(GovernanceError::NotFound(proposal_id).into())
    }
}
