use serde::{Deserialize, Serialize};
use validator::Validate;

use super::utils::Pagination;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProposalStatus {
    Pending,
    VotingPeriod,
    Passed,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProposalKind {
    Default,
    DefaultWithWasm,
    PgfSteward,
    PgfFunding,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ProposalQueryParams {
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
    pub status: Option<ProposalStatus>,
    pub kind: Option<ProposalKind>,
    pub pattern: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ProposalVotesQueryparams {
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
}
