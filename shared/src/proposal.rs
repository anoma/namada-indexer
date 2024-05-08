use crate::{block::Epoch, id::Id};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct GovernanceProposal {
    pub id: u64,
    pub content: String,
    pub r#type: String,
    pub proposal_code: Vec<u8>,
    pub author: Id,
    pub voting_start_epoch: Epoch,
    pub voting_end_epoch: Epoch,
    pub activation_epoch: Epoch
}