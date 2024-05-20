use serde::{Deserialize, Serialize};
use validator::Validate;

use super::utils::Pagination;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    VotingPeriod,
    Ended,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ProposalQueryParams {
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
    pub status: Option<ProposalStatus>,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct ProposalVotesQueryparams {
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
}

pub type ProposalSearchQueryParams = ProposalVotesQueryparams;
