use std::collections::BTreeSet;

use fake::Fake;
use namada_governance::ProposalType;
use namada_governance::storage::proposal::{
    AddRemove, PGFAction, PGFIbcTarget, PGFInternalTarget, PGFTarget,
    StorageProposal,
};
use namada_governance::utils::TallyType as NamadaTallyType;
use namada_ibc::core::host::types::identifiers::{ChannelId, PortId};
use namada_sdk::token::Amount;
use rand::distributions::{Distribution, Standard};
use subtle_encoding::hex;

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
    pub data: Option<String>,
    pub author: Id,
    pub voting_start_epoch: Epoch,
    pub voting_end_epoch: Epoch,
    pub activation_epoch: Epoch,
}

impl From<StorageProposal> for GovernanceProposal {
    fn from(proposal: StorageProposal) -> Self {
        let proposal_content_serialized =
            serde_json::to_string_pretty(&proposal.content).unwrap_or_default();

        Self {
            id: proposal.id,
            content: proposal_content_serialized,
            author: Id::from(proposal.author),
            r#type: proposal.r#type.into(),
            data: None,
            voting_start_epoch: proposal.voting_start_epoch.0 as Epoch,
            voting_end_epoch: proposal.voting_end_epoch.0 as Epoch,
            activation_epoch: proposal.activation_epoch.0 as Epoch,
        }
    }
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
            namada_core::address::gen_established_address("namada-indexer");
        let proposal_type: GovernanceProposalKind = rand::random();
        let proposal_data = match proposal_type {
            GovernanceProposalKind::PgfSteward => {
                let address_add = namada_core::address::gen_established_address(
                    "namada-indexer",
                );
                let address_remove =
                    namada_core::address::gen_established_address(
                        "namada-indexer",
                    );
                let mut data = BTreeSet::new();
                data.insert(AddRemove::Add(address_add));
                data.insert(AddRemove::Remove(address_remove));
                Some(serde_json::to_string(&data).unwrap())
            }
            GovernanceProposalKind::PgfFunding => {
                let address_retro =
                    namada_core::address::gen_established_address(
                        "namada-indexer",
                    );

                let mut data = BTreeSet::new();
                data.insert(PGFAction::Continuous(AddRemove::Add(
                    PGFTarget::Ibc(PGFIbcTarget {
                        target: (15..16).fake::<String>(),
                        amount: Amount::from_u64((0..1000).fake::<u64>()),
                        port_id: PortId::new((5..6).fake::<String>()).unwrap(),
                        channel_id: ChannelId::new((5..6).fake::<u64>()),
                    }),
                )));
                data.insert(PGFAction::Retro(PGFTarget::Internal(
                    PGFInternalTarget {
                        target: address_retro,
                        amount: Amount::from_u64((0..1000).fake::<u64>()),
                    },
                )));
                Some(serde_json::to_string(&data).unwrap())
            }
            GovernanceProposalKind::Default => None,
            GovernanceProposalKind::DefaultWithWasm => {
                let data = fake::vec![u8; 100..1000];
                let hex_encoded =
                    String::from_utf8_lossy(&hex::encode(data)).to_string();
                Some(hex_encoded)
            }
        };
        let voting_start_epoch = (1..1000).fake::<u32>();
        let voting_end_epoch =
            (voting_start_epoch..voting_start_epoch + 20).fake::<u32>();
        let activation_epoch = voting_end_epoch + 3 + (0..10).fake::<u32>();

        Self {
            id: proposal_id,
            r#type: proposal_type,
            data: proposal_data,
            author: Id::Account(address.to_string()),
            voting_start_epoch,
            voting_end_epoch,
            activation_epoch,
            content: "Lorem ipsum dolor sit amet, consectetur adipiscing \
                      elit. Nullam purus tellus, mollis in nisi sed, laoreet \
                      scelerisque ante. Mauris at odio in magna ullamcorper \
                      lobortis. Nunc quis leo vulputate, tristique metus sed, \
                      laoreet nulla. Phasellus convallis tincidunt ante, in \
                      auctor massa. Praesent porta metus a interdum \
                      scelerisque. Pellentesque habitant morbi tristique \
                      senectus et netus et malesuada fames ac turpis egestas. \
                      Cras non vulputate nisl. Nulla placerat rhoncus \
                      consequat. Praesent orci sapien, tincidunt maximus \
                      dictum id, elementum quis urna. Suspendisse finibus \
                      ante at nulla consectetur, eu congue velit semper. \
                      Donec eget pulvinar enim. Vestibulum molestie semper \
                      magna id porttitor. Class aptent taciti sociosqu ad \
                      litora torquent per conubia nostra, per inceptos \
                      himenaeos. Quisque ornare, urna et convallis elementum, \
                      massa leo aliquet risus, eu varius ipsum dolor nec \
                      nunc. Praesent magna quam, porta nec justo non, \
                      vestibulum ultricies nisl.Aenean id accumsan lorem. \
                      Suspendisse vestibulum viverra nisi, id pharetra ex \
                      maximus a. Vestibulum a vulputate nisi. Praesent \
                      hendrerit blandit diam eget gravida. Sed non velit \
                      egestas, egestas magna ultricies, porttitor libero. \
                      Nullam facilisis arcu a condimentum scelerisque. \
                      Curabitur pharetra blandit dui, et pellentesque felis \
                      consequat non. Lorem ipsum dolor sit amet, consectetur \
                      adipiscing elit. Proin id vestibulum eros. Mauris \
                      ullamcorper lacinia lorem, aliquam feugiat orci \
                      consequat faucibus. Cras sit amet ex nulla. Nullam \
                      vestibulum nisl quis nunc iaculis sagittis. Integer ut \
                      porttitor risus. Duis vel lectus ante."
                .to_string(),
        }
    }
}

impl GovernanceProposalStatus {
    pub fn fake(proposal_id: u64) -> Self {
        Self {
            id: proposal_id,
            result: rand::random(),
            yay_votes: (0..10000).fake::<u64>().to_string(),
            nay_votes: (0..10000).fake::<u64>().to_string(),
            abstain_votes: (0..10000).fake::<u64>().to_string(),
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

impl Distribution<GovernanceProposalResult> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> GovernanceProposalResult {
        match rng.gen_range(0..=3) {
            0 => GovernanceProposalResult::Passed,
            1 => GovernanceProposalResult::Pending,
            2 => GovernanceProposalResult::VotingPeriod,
            _ => GovernanceProposalResult::Rejected,
        }
    }
}

pub enum TallyType {
    TwoFifths,
    OneHalfOverOneThird,
    LessOneHalfOverOneThirdNay,
}

// TODO: copied from namada for time being
impl TallyType {
    pub fn fake() -> Self {
        rand::random()
    }

    pub fn from(
        proposal_type: &GovernanceProposalKind,
        is_steward: bool,
    ) -> Self {
        let namada_proposal_type = match proposal_type {
            GovernanceProposalKind::Default => ProposalType::Default,
            GovernanceProposalKind::DefaultWithWasm => {
                ProposalType::DefaultWithWasm(namada_sdk::hash::Hash([0; 32]))
            }
            GovernanceProposalKind::PgfSteward => {
                ProposalType::PGFSteward(BTreeSet::new())
            }
            GovernanceProposalKind::PgfFunding => {
                ProposalType::PGFPayment(BTreeSet::new())
            }
        };

        let tally_type =
            NamadaTallyType::from(namada_proposal_type, is_steward);

        TallyType::from_namada_tally_type(tally_type)
    }

    pub fn from_namada_tally_type(tally_type: NamadaTallyType) -> Self {
        // With this match we couple the types which is desired
        match tally_type {
            NamadaTallyType::TwoFifths => TallyType::TwoFifths,
            NamadaTallyType::OneHalfOverOneThird => {
                TallyType::OneHalfOverOneThird
            }
            NamadaTallyType::LessOneHalfOverOneThirdNay => {
                TallyType::LessOneHalfOverOneThirdNay
            }
        }
    }
}

impl Distribution<TallyType> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> TallyType {
        match rng.gen_range(0..=2) {
            0 => TallyType::TwoFifths,
            1 => TallyType::OneHalfOverOneThird,
            _ => TallyType::LessOneHalfOverOneThirdNay,
        }
    }
}
