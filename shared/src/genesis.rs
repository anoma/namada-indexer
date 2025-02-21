use serde::{Deserialize, Serialize};
use tendermint::{AppHash, Time, chain, consensus, validator};
use tendermint_rpc::dialect::LatestDialect;
use tendermint_rpc::request::RequestMessage;
use tendermint_rpc::{
    Method, Request as TendermintRequest, Response as TendermintResponse,
    SimpleRequest,
};

#[derive(Clone, Debug)]
pub struct Genesis {
    pub genesis_time: i64,
    pub chain_id: String,
}

impl From<GenesisParams<serde_json::Value>> for Genesis {
    fn from(genesis: GenesisParams<serde_json::Value>) -> Self {
        Self {
            genesis_time: genesis.genesis_time.unix_timestamp(),
            chain_id: String::from(genesis.chain_id),
        }
    }
}

// Overriding genesis response as app_state is missing and deserialization fails
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisParams<AppState = serde_json::Value> {
    /// Time of genesis
    pub genesis_time: Time,

    /// Chain ID
    pub chain_id: chain::Id,

    #[serde(skip)]
    pub initial_height: i64,

    pub consensus_params: consensus::Params,

    /// Validators
    #[serde(skip)]
    pub validators: Vec<validator::Info>,

    /// App hash
    #[serde(skip)]
    pub app_hash: AppHash,

    #[serde(skip)]
    pub app_state: AppState,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Genesis data
    pub genesis: GenesisParams,
}

impl TendermintResponse for Response {}

#[derive(Serialize, Deserialize, Default)]
pub struct GenesisRequest;

impl RequestMessage for GenesisRequest {
    fn method(&self) -> Method {
        Method::Genesis
    }
}

impl TendermintRequest for GenesisRequest {
    type Response = Response;
}

impl SimpleRequest<LatestDialect> for GenesisRequest {
    type Output = Self::Response;
}
