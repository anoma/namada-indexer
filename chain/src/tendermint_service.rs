use anyhow::Context;
use tendermint_rpc::{
    endpoint::block::Response as TendermintBlockResponse,
    endpoint::block_results::Response as TendermintBlockResultResponse, Client,
    HttpClient,
};

pub async fn query_raw_block_at_height(
    client: &HttpClient,
    height: u32,
) -> anyhow::Result<TendermintBlockResponse> {
    client
        .block(height)
        .await
        .context("Failed to query CometBFT's last committed height")
}

pub async fn query_raw_block_results_at_height(
    client: &HttpClient,
    height: u32,
) -> anyhow::Result<TendermintBlockResultResponse> {
    client
        .block_results(height)
        .await
        .context("Failed to query CometBFT's block results")
}
