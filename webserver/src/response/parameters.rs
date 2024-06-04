use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub unbonding_length: u64,
    pub pipeline_length: u64,
    pub epochs_per_year: u64,
}
