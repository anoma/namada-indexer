use orm::balances::BalanceDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBalance {
    pub token_address: String,
    pub min_denom_amount: String,
}

impl From<BalanceDb> for AddressBalance {
    fn from(value: BalanceDb) -> Self {
        Self {
            token_address: value.token,
            min_denom_amount: value.raw_amount.to_string(),
        }
    }
}
