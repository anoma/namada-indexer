use fake::Fake;
use namada_governance::ProposalType;
use rand::distributions::{Distribution, Standard};

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

impl GovernanceProposal {
    pub fn fake(proposal_id: u64) -> Self {
        let address =
            namada_core::address::gen_established_address("123");
        let proposal_type: GovernanceProposalKind = rand::random();
        let proposal_data = vec![].to_vec();
        let voting_start_epoch = (1..1000).fake::<u32>();
        let voting_end_epoch =
            (voting_start_epoch..voting_start_epoch + 20).fake::<u32>();
        let activation_epoch = voting_end_epoch + 3 + (0..10).fake::<u32>();

        Self {
            id: proposal_id,
            r#type: proposal_type,
            data: Some(proposal_data),
            author: Id::Account(address.to_string()),
            voting_start_epoch,
            voting_end_epoch,
            activation_epoch,
            content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam purus tellus, mollis in nisi sed, laoreet scelerisque ante. Mauris at odio in magna ullamcorper lobortis. Nunc quis leo vulputate, tristique metus sed, laoreet nulla. Phasellus convallis tincidunt ante, in auctor massa. Praesent porta metus a interdum scelerisque. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Cras non vulputate nisl. Nulla placerat rhoncus consequat. Praesent orci sapien, tincidunt maximus dictum id, elementum quis urna. Suspendisse finibus ante at nulla consectetur, eu congue velit semper. Donec eget pulvinar enim. Vestibulum molestie semper magna id porttitor. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Quisque ornare, urna et convallis elementum, massa leo aliquet risus, eu varius ipsum dolor nec nunc. Praesent magna quam, porta nec justo non, vestibulum ultricies nisl.Aenean id accumsan lorem. Suspendisse vestibulum viverra nisi, id pharetra ex maximus a. Vestibulum a vulputate nisi. Praesent hendrerit blandit diam eget gravida. Sed non velit egestas, egestas magna ultricies, porttitor libero. Nullam facilisis arcu a condimentum scelerisque. Curabitur pharetra blandit dui, et pellentesque felis consequat non. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin id vestibulum eros. Mauris ullamcorper lacinia lorem, aliquam feugiat orci consequat faucibus. Cras sit amet ex nulla. Nullam vestibulum nisl quis nunc iaculis sagittis. Integer ut porttitor risus. Duis vel lectus ante.".to_string()
        }
    }
}

impl Distribution<GovernanceProposalKind> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> GovernanceProposalKind {
        match rng.gen_range(0..=3) {
            0 => GovernanceProposalKind::DefaultWithWasm,
            1 => GovernanceProposalKind::PgfSteward,
            2 => GovernanceProposalKind::PgfFunding,
            _ => GovernanceProposalKind::Default,
        }
    }
}
