use std::fmt::Display;

use namada_core::time::DateTimeUtc;
use orm::governance_proposal::{
    GovernanceProposalDb, GovernanceProposalKindDb, GovernanceProposalResultDb,
    GovernanceProposalTallyTypeDb,
};
use orm::governance_votes::{GovernanceProposalVoteDb, GovernanceVoteKindDb};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProposalType {
    Default,
    DefaultWithWasm,
    PgfSteward,
    PgfFunding,
}

impl Display for ProposalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalType::Default => write!(f, "default"),
            ProposalType::DefaultWithWasm => write!(f, "default_with_wasm"),
            ProposalType::PgfSteward => write!(f, "pgf_steward"),
            ProposalType::PgfFunding => write!(f, "pgf_funding"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TallyType {
    TwoThirds,
    OneHalfOverOneThird,
    LessOneHalfOverOneThirdNay,
}

impl Display for TallyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TallyType::TwoThirds => write!(f, "two_thirds"),
            TallyType::OneHalfOverOneThird => {
                write!(f, "one_half_over_one_third")
            }
            TallyType::LessOneHalfOverOneThirdNay => {
                write!(f, "less_one_half_over_one_third_nay")
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VoteType {
    Yay,
    Nay,
    Abstain,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProposalStatus {
    Pending,
    Rejected,
    Passed,
    Voting,
    Unknown,
}

impl Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Pending => write!(f, "Pending"),
            ProposalStatus::Rejected => write!(f, "Rejected"),
            ProposalStatus::Passed => write!(f, "Passed"),
            ProposalStatus::Voting => write!(f, "Voting"),
            ProposalStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: u64,
    pub content: String,
    pub r#type: ProposalType,
    pub tally_type: TallyType,
    pub data: Option<String>,
    pub author: String,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub activation_epoch: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub current_time: i64,
    pub status: ProposalStatus,
    pub yay_votes: String,
    pub nay_votes: String,
    pub abstain_votes: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalVote {
    pub proposal_id: u64,
    pub vote: VoteType,
    pub voter_address: String,
}

// TODO: read it from storage later
const CONSENSUS_TIME_IN_SEC: i64 = 10;
const MIN_NUMBER_OF_BLOCKS: i64 = 4;

impl Proposal {
    pub fn from_proposal_db(
        value: GovernanceProposalDb,
        current_epoch: i32,
        current_block: i32,
    ) -> Self {
        let seconds_since_beginning =
            i64::from(current_block) * CONSENSUS_TIME_IN_SEC;
        let seconds_until_end = i64::from(value.end_epoch - current_epoch)
            * MIN_NUMBER_OF_BLOCKS
            * CONSENSUS_TIME_IN_SEC;

        let time_now = DateTimeUtc::now().0.timestamp();
        let start_time = time_now - seconds_since_beginning;
        let end_time = time_now + seconds_until_end;

        Self {
            id: value.id as u64,
            content: value.content,
            r#type: match value.kind {
                GovernanceProposalKindDb::PgfSteward => {
                    ProposalType::PgfSteward
                }
                GovernanceProposalKindDb::PgfFunding => {
                    ProposalType::PgfFunding
                }
                GovernanceProposalKindDb::Default => ProposalType::Default,
                GovernanceProposalKindDb::DefaultWithWasm => {
                    ProposalType::DefaultWithWasm
                }
            },
            tally_type: match value.tally_type {
                GovernanceProposalTallyTypeDb::TwoThirds => {
                    TallyType::TwoThirds
                }
                GovernanceProposalTallyTypeDb::OneHalfOverOneThird => {
                    TallyType::OneHalfOverOneThird
                }
                GovernanceProposalTallyTypeDb::LessOneHalfOverOneThirdNay => {
                    TallyType::LessOneHalfOverOneThirdNay
                }
            },
            data: value.data,
            author: value.author,
            start_epoch: value.start_epoch as u64,
            end_epoch: value.end_epoch as u64,
            activation_epoch: value.activation_epoch as u64,

            start_time,
            end_time,
            current_time: time_now,

            status: match value.result {
                GovernanceProposalResultDb::Passed => ProposalStatus::Passed,
                GovernanceProposalResultDb::Rejected => {
                    ProposalStatus::Rejected
                }
                GovernanceProposalResultDb::Pending => ProposalStatus::Pending,
                GovernanceProposalResultDb::Unknown => ProposalStatus::Unknown,
                GovernanceProposalResultDb::VotingPeriod => {
                    ProposalStatus::Voting
                }
            },
            yay_votes: value.yay_votes,
            nay_votes: value.nay_votes,
            abstain_votes: value.abstain_votes,
        }
    }
}

impl From<GovernanceProposalVoteDb> for ProposalVote {
    fn from(value: GovernanceProposalVoteDb) -> Self {
        Self {
            proposal_id: value.proposal_id as u64,
            vote: match value.kind {
                GovernanceVoteKindDb::Nay => VoteType::Nay,
                GovernanceVoteKindDb::Yay => VoteType::Yay,
                GovernanceVoteKindDb::Abstain => VoteType::Abstain,
            },
            voter_address: value.voter_address,
        }
    }
}
