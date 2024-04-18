use namada_sdk::token::Amount as NamadaAmount;

pub type Address = String;

#[derive(Debug, Clone)]
pub struct Amount(pub String);

impl From<NamadaAmount> for Amount {
    fn from(amount: NamadaAmount) -> Amount {
        Amount(amount.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub owner: Address,
    pub amount: Amount,
}

pub type Balances = Vec<Balance>;
