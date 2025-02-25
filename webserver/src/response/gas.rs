use orm::gas::GasPriceDb;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPrice {
    pub token: String,
    pub min_denom_amount: String,
}

impl From<GasPriceDb> for GasPrice {
    fn from(value: GasPriceDb) -> Self {
        Self {
            token: value.token,
            min_denom_amount: value.amount.to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasEstimate {
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub total_estimates: u64,
}
