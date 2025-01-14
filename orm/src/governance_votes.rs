use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use shared::vote::{GovernanceVote, ProposalVoteKind};

use crate::schema::governance_votes;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::VoteKind"]
pub enum GovernanceVoteKindDb {
    Nay,
    Yay,
    Abstain,
    Unknown,
}

impl From<ProposalVoteKind> for GovernanceVoteKindDb {
    fn from(value: ProposalVoteKind) -> Self {
        match value {
            ProposalVoteKind::Nay => Self::Nay,
            ProposalVoteKind::Yay => Self::Yay,
            ProposalVoteKind::Abstain => Self::Abstain,
            ProposalVoteKind::Unknown => Self::Unknown,
        }
    }
}

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = governance_votes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalVoteDb {
    pub id: i32,
    pub voter_address: String,
    pub kind: GovernanceVoteKindDb,
    pub proposal_id: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = governance_votes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalVoteInsertDb {
    pub voter_address: String,
    pub kind: GovernanceVoteKindDb,
    pub proposal_id: i32,
}

impl GovernanceProposalVoteInsertDb {
    pub fn from_governance_vote(vote: GovernanceVote) -> Self {
        Self {
            voter_address: vote.address.to_string(),
            kind: vote.vote.into(),
            proposal_id: vote.proposal_id as i32,
        }
    }
}
