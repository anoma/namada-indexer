use serde::{Deserialize, Serialize};

use super::chain::Token;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBalance {
    pub token: Token,
    pub min_denom_amount: String,
}
