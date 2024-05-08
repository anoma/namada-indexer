use namada_governance::ProposalVote;
use serde::{Deserialize, Serialize};

use crate::id::Id;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum ProposalVoteKind {
    Nay,
    Yay,
    Abstain,
}

impl From<ProposalVote> for ProposalVoteKind {
    fn from(value: ProposalVote) -> Self {
        match value {
            ProposalVote::Nay => Self::Nay,
            ProposalVote::Yay => Self::Yay,
            ProposalVote::Abstain => Self::Abstain,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceVote {
    pub proposal_id: u64,
    pub vote: ProposalVoteKind,
    pub address: Id,
}
