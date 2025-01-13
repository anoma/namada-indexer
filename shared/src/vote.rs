use namada_governance::ProposalVote;
use rand::distributions::{Distribution, Standard};
use serde::{Deserialize, Serialize};

use crate::id::Id;

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum ProposalVoteKind {
    Nay,
    Yay,
    Abstain,
    Unknown,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GovernanceVote {
    pub proposal_id: u64,
    pub vote: ProposalVoteKind,
    pub address: Id,
}

impl GovernanceVote {
    pub fn fake(proposal_id: u64) -> Self {
        let address =
            namada_core::address::gen_established_address("namada-indexer");

        let vote: ProposalVoteKind = rand::random();
        Self {
            proposal_id,
            vote,
            address: Id::Account(address.to_string()),
        }
    }
}

impl Distribution<ProposalVoteKind> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> ProposalVoteKind {
        match rng.gen_range(0..=2) {
            0 => ProposalVoteKind::Nay,
            1 => ProposalVoteKind::Yay,
            _ => ProposalVoteKind::Abstain,
        }
    }
}
