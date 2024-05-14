use crate::balance::Amount;
use crate::id::Id;
use crate::utils::DelegationPair;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reward {
    pub delegation_pair: DelegationPair,
    pub amount: Amount,
}

impl Reward {
    pub fn fake() -> Self {
        let validator_address =
            namada_core::address::gen_established_address("namada-indexer");
        let delegator_address =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            delegation_pair: DelegationPair {
                validator_address: Id::Account(validator_address.to_string()),
                delegator_address: Id::Account(delegator_address.to_string()),
            },
            amount: Amount::fake(),
        }
    }
}
