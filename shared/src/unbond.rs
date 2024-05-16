use fake::Fake;

use crate::balance::Amount;
use crate::block::Epoch;
use crate::id::Id;

// TODO: maybe reuse bond with Option<Amount> instead of Amount
#[derive(Debug, Clone)]
pub struct UnbondAddresses {
    pub source: Id,
    pub validator: Id,
}

#[derive(Debug, Clone)]
pub struct Unbond {
    pub epoch: Epoch,
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
            epoch: (1..3).fake::<u32>(),
            withdraw_at: (3..10).fake::<u32>(),
        }
    }
}

pub type Unbonds = Vec<Unbond>;
