use orm::balances::BalanceDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBalance {
    pub token_address: String,
    pub balances: u64,
}

impl From<BalanceDb> for AddressBalance {
    fn from(value: BalanceDb) -> Self {
        Self {
            token_address: value.token,
            balances: 10, // value.raw_amount,
        }
    }
}
