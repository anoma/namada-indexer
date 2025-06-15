pub mod config;
pub mod functions;
pub mod namada;
pub mod utils;

use clap::Parser;
use namada_sdk::tendermint_rpc::HttpClient;
use namada_sdk::tendermint_rpc::client::CompatMode;

use crate::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    let client = HttpClient::builder(config.tendermint_url.parse().unwrap())
        .compat_mode(CompatMode::V0_37)
        .build()
        .unwrap();

    if config.fix_tx {
        functions::fix::fix(&client).await?;
    } else if config.deserialize_tx && config.block_height.is_some() {
        let block_height = config.block_height.unwrap();
        // functions::deserialize::deserialize_tx(&client, 2410415).await?;
        // failing
        functions::deserialize_block::deserialize_tx(&client, block_height)
            .await?;
    } else if config.query_account && config.address.is_some() {
        let address = config.address.as_ref().unwrap();
        functions::query_account::query_account(&client, address).await?;
    } else {
        println!("No action specified.");
    }

    Ok(())
}
