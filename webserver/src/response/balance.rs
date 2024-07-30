use serde::{Deserialize, Serialize};
use shared::balance::{Balance, DenominatedAmount};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBalance {
    pub token_address: String,
    pub balance: String,
}

impl From<Balance> for AddressBalance {
    fn from(value: Balance) -> Self {
        Self {
            token_address: value.token.to_string(),
            // TODO: temporary solution as we only store NAM balances
            balance: DenominatedAmount::native(value.amount.clone())
                .to_string_precise(),
        }
    }
}
