use crate::balance::Amount;
use crate::utils::DelegationPair;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reward {
    pub delegation_pair: DelegationPair,
    pub amount: Amount,
}
