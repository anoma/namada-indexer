use crate::id::Id;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BalanceChange {
    pub address: Id,
    pub token: Token,
}

impl BalanceChange {
    pub fn new(address: Id, token: Token) -> Self {
        Self { address, token }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct GovernanceProposalShort {
    pub id: u64,
    pub voting_start_epoch: u64,
    pub voting_end_epoch: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DelegationPair {
    pub validator_address: Id,
    pub delegator_address: Id,
}
