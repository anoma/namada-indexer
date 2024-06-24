use orm::parameters::ParametersDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub unbonding_length: String,
    pub pipeline_length: String,
    pub epochs_per_year: String,
    pub apr: String,
    pub native_token_address: String,
    pub chain_id: String,
    pub genesis_time: String,
    pub min_duration: String,
}

impl From<ParametersDb> for Parameters {
    fn from(parameters: ParametersDb) -> Self {
        Self {
            unbonding_length: parameters.unbonding_length.to_string(),
            pipeline_length: parameters.pipeline_length.to_string(),
            epochs_per_year: parameters.epochs_per_year.to_string(),
            apr: parameters.apr,
            native_token_address: parameters.native_token_address,
            chain_id: parameters.chain_id,
            genesis_time: parameters.genesis_time.to_string(),
            min_duration: parameters.min_duration.to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcUrl {
    pub url: String,
}
