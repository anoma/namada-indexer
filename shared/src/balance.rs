use fake::Fake;
use namada_sdk::token::Amount as NamadaAmount;

use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Amount(pub [u64; 4]);

impl Amount {
    pub fn fake() -> Self {
        let one = (u64::MIN..u64::MAX).fake::<u64>();
        let two = (u64::MIN..u64::MAX).fake::<u64>();
        let three = (u64::MIN..u64::MAX).fake::<u64>();
        let four = (u64::MIN..u64::MAX).fake::<u64>();
        Self([one, two, three, four])
    }
}

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

impl Balance {
    pub fn fake() -> Self {
        let address =
            namada_core::address::gen_established_address("namada-indexer");
        let token_address = namada_core::address::gen_established_address("namada-indexer");

        Self {
            owner: Id::Account(address.to_string()),
            token: Id::Account(token_address.to_string()),
            amount: Amount::fake(),
        }
    }
}
