use serde::{Deserialize, Serialize};

#[derive(Hash, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GasCost {
    pub token_address: String,
    pub amount: String,
}
