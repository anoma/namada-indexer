use namada_governance::ProposalType;

use crate::block::Epoch;
use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GovernanceProposalKind {
    PgfSteward,
    PgfFunding,
    Default,
    DefaultWithWasm,
}

impl From<ProposalType> for GovernanceProposalKind {
    fn from(value: ProposalType) -> Self {
        match value {
            ProposalType::Default => Self::Default,
            ProposalType::DefaultWithWasm(_) => Self::DefaultWithWasm,
            ProposalType::PGFSteward(_) => Self::PgfSteward,
            ProposalType::PGFPayment(_) => Self::PgfFunding,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GovernanceProposalResult {
    Passed,
    Rejected,
    VotingPeriod,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GovernanceProposal {
    pub id: u64,
    pub content: String,
    pub r#type: GovernanceProposalKind,
    pub data: Option<Vec<u8>>,
    pub author: Id,
    pub voting_start_epoch: Epoch,
    pub voting_end_epoch: Epoch,
    pub activation_epoch: Epoch,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GovernanceProposalStatus {
    pub id: u64,
    pub result: GovernanceProposalResult,
    pub yay_votes: String,
    pub nay_votes: String,
    pub abstain_votes: String,
}