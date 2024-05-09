use crate::{balance::Amount, utils::DelegationPair};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reward {
    pub delegation_pair: DelegationPair,
    pub amount: Amount,
}
