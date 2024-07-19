use orm::parameters::ParametersDb;
use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeJSONValue;

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
    pub min_num_of_blocks: String,
    pub checksums: SerdeJSONValue,
    pub epoch_switch_blocks_delay: String,
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
            min_num_of_blocks: parameters.min_num_of_blocks.to_string(),
            checksums: parameters.checksums,
            epoch_switch_blocks_delay: parameters
                .epoch_switch_blocks_delay
                .to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcUrl {
    pub url: String,
}
