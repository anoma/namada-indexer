use crate::id::Id;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BalanceChange {
    pub address: Id,
    pub token: Id,
}

impl BalanceChange {
    pub fn new(address: Id, token: Id) -> Self {
        Self { address, token }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct GovernanceProposalShort {
    pub id: u64,
    pub voting_start_epoch: u64,
    pub voting_end_epoch: u64,
}
