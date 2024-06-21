use std::fmt::Display;

use fake::Fake;
use namada_sdk::token::Amount as NamadaAmount;

use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Amount(NamadaAmount);

impl From<NamadaAmount> for Amount {
    fn from(amount: NamadaAmount) -> Amount {
        Amount(amount)
    }
}

impl Amount {
    pub fn fake() -> Self {
        Self(NamadaAmount::from_u64((0..10000000).fake::<u64>()))
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub owner: Id,
    pub token: Id,
    pub amount: Amount,
}

pub type Balances = Vec<Balance>;

impl Balance {
    pub fn fake() -> Self {
        let address =
            namada_core::address::gen_established_address("namada-indexer");
        let token_address =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            owner: Id::Account(address.to_string()),
            token: Id::Account(token_address.to_string()),
            amount: Amount::fake(),
        }
    }
}
