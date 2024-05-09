use namada_sdk::token::Amount as NamadaAmount;

pub type Address = String;

#[derive(Debug, Clone)]
pub struct Amount(pub [u64; 4]);

impl From<NamadaAmount> for Amount {
    fn from(amount: NamadaAmount) -> Amount {
        Amount(amount.raw_amount().0)
    }
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub owner: Address,
    pub token: Address,
    pub amount: Amount,
}

pub type Balances = Vec<Balance>;