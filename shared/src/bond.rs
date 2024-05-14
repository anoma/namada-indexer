use crate::balance::Amount;
use crate::block::Epoch;
use crate::id::Id;

// TODO: maybe reuse bond with Option<Amount> instead of Amount
#[derive(Debug, Clone)]
pub struct BondAddresses {
    pub source: Id,
    pub target: Id,
}

#[derive(Debug, Clone)]
pub struct Bond {
    pub source: Id,
    pub target: Id,
    pub amount: Amount,
}

#[derive(Debug, Clone)]
pub struct Bonds {
    pub epoch: Epoch,
    pub values: Vec<Bond>,
}

impl Bond {
    pub fn fake(validator_address: Id) -> Self {
        let source_address =
            namada_core::address::gen_established_address("namada-indexer");
        Self {
            source: Id::Account(source_address.to_string()),
            target: validator_address,
            amount: Amount::fake(),
        }
    }
}

impl Bonds {
    pub fn new(bonds: Vec<Bond>, epoch: Epoch) -> Self {
        Self {
            epoch,
            values: bonds,
        }
    }
}
