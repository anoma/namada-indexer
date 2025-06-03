use orm::parameters::ParametersDb;
use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeJSONValue;
use shared::token::Token as SharedToken;

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
    pub max_block_time: String,
    pub checksums: SerdeJSONValue,
    pub epoch_switch_blocks_delay: String,
    pub cubic_slashing_window_length: String,
    pub duplicate_vote_min_slash_rate: String,
    pub light_client_attack_min_slash_rate: String,
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
            max_block_time: parameters.max_block_time.to_string(),
            checksums: parameters.checksums,
            epoch_switch_blocks_delay: parameters
                .epoch_switch_blocks_delay
                .to_string(),
            cubic_slashing_window_length: parameters
                .cubic_slashing_window_length
                .to_string(),
            duplicate_vote_min_slash_rate: parameters
                .duplicate_vote_min_slash_rate
                .to_string(),
            light_client_attack_min_slash_rate: parameters
                .light_client_attack_min_slash_rate
                .to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcUrl {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastProcessedBlock {
    pub block: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastProcessedEpoch {
    pub epoch: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeToken {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IbcToken {
    pub address: String,
    pub trace: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Token {
    Native(NativeToken),
    Ibc(IbcToken),
}

impl From<SharedToken> for Token {
    fn from(value: SharedToken) -> Self {
        match value {
            SharedToken::Native(token) => Token::Native(NativeToken {
                address: token.to_string(),
            }),
            SharedToken::Ibc(token) => Token::Ibc(IbcToken {
                address: token.address.to_string(),
                trace: token.trace.unwrap_or_default().to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSupply {
    pub address: String,
    pub total_supply: String,
    pub effective_supply: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CirculatingSupply {
    pub circulating_supply: String,
}
