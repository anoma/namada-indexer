use std::convert::identity;
use std::sync::Arc;

use anyhow::Context;
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use cometbft::app_state::AppState;
use cometbft::config::AppConfig;
use cometbft::repository::cometbft as cometbft_repo;
use cometbft::services::{
    db as db_service, namada as namada_service,
    tendermint as tendermint_service,
};
use deadpool_diesel::postgres::Object;
use orm::migrations::run_migrations;
use shared::crawler::crawl;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tendermint_rpc::client::CompatMode;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    config.log.init();

    let client = Arc::new(
        HttpClient::builder(config.tendermint_url.as_str().parse().unwrap())
            .compat_mode(CompatMode::V0_37)
            .build()
            .unwrap(),
    );

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    let crawler_state_height = db_service::get_cometbft_crawler_state(&conn)
        .await
        .map(|s| s.last_processed_block)
        .unwrap_or_default();
    let transaction_state_height =
        db_service::get_transaction_crawler_state(&conn)
            .await
            .map(|s| s.last_processed_block)
            .unwrap_or_default();
    let chain_state_height = db_service::get_chain_crawler_state(&conn)
        .await
        .map(|s| s.last_processed_block)
        .unwrap_or_default();

    let block_height = std::cmp::max(
        std::cmp::max(
            std::cmp::min(chain_state_height, transaction_state_height),
            crawler_state_height,
        ),
        1,
    );

    crawl(
        move |block_height| {
            crawling_fn(block_height, client.clone(), conn.clone())
        },
        block_height,
        None,
    )
    .await
}

async fn crawling_fn(
    block_height: u32,
    client: Arc<HttpClient>,
    conn: Arc<Object>,
) -> Result<(), MainError> {
    let should_process = can_process(block_height, client.clone()).await?;

    if !should_process {
        let timestamp = Utc::now().naive_utc();
        update_crawler_timestamp(&conn, timestamp).await?;

        tracing::trace!(
            block = block_height,
            "Block does not exist yet, waiting...",
        );

        return Err(MainError::NoAction);
    }

    let start = Instant::now();

    tracing::debug!(block = block_height, "Query block...");
    let encoded_block =
        tendermint_service::query_raw_block_at_height(&client, block_height)
            .await
            .into_rpc_error()
            .map(|block| serde_json::to_string(&block).unwrap())?;
    tracing::debug!(block = block_height, "Query block results...");
    let encoded_block_result =
        tendermint_service::query_raw_block_results_at_height(
            &client,
            block_height,
        )
        .await
        .into_rpc_error()
        .map(|events| serde_json::to_string(&events).unwrap())?;

    let first_checkpoint = Instant::now();

    tracing::info!(
        block_height = block_height,
        time_taken = first_checkpoint.duration_since(start).as_secs_f64(),
        "Queried block successfully",
    );

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                cometbft_repo::upsert_block(
                    transaction_conn,
                    block_height,
                    encoded_block,
                    encoded_block_result,
                )?;

                cometbft_repo::insert_crawler_state(
                    transaction_conn,
                    shared::crawler_state::BlockCrawlerState {
                        last_processed_block: block_height,
                        timestamp: chrono::Utc::now().timestamp(),
                    },
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()?;

    let second_checkpoint = Instant::now();

    tracing::info!(
        block = block_height,
        time_taken = second_checkpoint
            .duration_since(first_checkpoint)
            .as_secs_f64(),
        "Inserted block into database"
    );

    Ok(())
}

async fn can_process(
    block_height: u32,
    client: Arc<HttpClient>,
) -> Result<bool, MainError> {
    let last_block_height =
        namada_service::get_last_block(&client).await.map_err(|e| {
            tracing::error!(
                "Failed to query Namada's last committed block: {}",
                e
            );
            MainError::RpcError
        })?;

    Ok(last_block_height >= block_height)
}

async fn update_crawler_timestamp(
    conn: &Object,
    timestamp: NaiveDateTime,
) -> Result<(), MainError> {
    conn.interact(move |transaction_conn| {
        cometbft_repo::update_timestamp(transaction_conn, timestamp)?;

        anyhow::Ok(())
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Insert crawler state error")
    .into_db_error()
}
