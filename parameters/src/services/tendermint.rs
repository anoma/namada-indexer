use namada_sdk::tendermint_rpc::{Client, HttpClient};
use shared::genesis::{Genesis, GenesisParams, GenesisRequest};

pub async fn query_genesis(client: &HttpClient) -> anyhow::Result<Genesis> {
    let genesis_params: GenesisParams =
        client.perform(GenesisRequest).await?.genesis;
 
    Ok(Genesis::from(genesis_params))
}
