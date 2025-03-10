use fake::Fake;

use crate::balance::Amount;
use crate::block::Epoch;
use crate::id::Id;

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub struct BondAddresses {
    pub source: Id,
    pub target: Id,
}

#[derive(Debug, Clone)]
pub struct Bond {
    pub source: Id,
    pub target: Id,
    pub amount: Amount,
    pub start: Epoch,
}

impl Bond {
    pub fn fake(validator_address: Id) -> Self {
        let source_address =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            source: Id::Account(source_address.to_string()),
            target: validator_address,
            amount: Amount::fake(),
            start: (1..1000).fake::<u32>(),
        }
    }
}

pub type Bonds = Vec<Bond>;

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub struct UnbondAddresses {
    pub source: Id,
    pub validator: Id,
}

#[derive(Debug, Clone)]
pub struct Unbond {
    pub source: Id,
    pub target: Id,
    pub amount: Amount,
    pub withdraw_at: Epoch,
}

impl Unbond {
    pub fn fake(validator_address: Id) -> Self {
        let source_address =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            source: Id::Account(source_address.to_string()),
            target: validator_address,
            amount: Amount::fake(),
            withdraw_at: (3..10).fake::<u32>(),
        }
    }
}

pub type Unbonds = Vec<Unbond>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redelegation {
    pub delegator: Id,
    pub validator: Id,
    pub epoch: Epoch,
}

impl Redelegation {
    pub fn fake(validator_address: Id) -> Self {
        let delegator =
            namada_core::address::gen_established_address("delegator");

        Self {
            delegator: Id::from(delegator),
            validator: validator_address,
            epoch: (1..1000).fake::<u32>(),
        }
    }
}

pub type Redelegations = Vec<Redelegation>;
