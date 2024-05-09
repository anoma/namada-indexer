use namada_sdk::token::Amount as NamadaAmount;

use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Amount(pub [u64; 4]);

impl From<NamadaAmount> for Amount {
    fn from(amount: NamadaAmount) -> Amount {
        Amount(amount.raw_amount().0)
    }
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub owner: Id,
    pub token: Id,
    pub amount: Amount,
}

pub type Balances = Vec<Balance>;
