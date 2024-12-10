use std::convert::identity;
use std::sync::Arc;

use anyhow::Context;
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use deadpool_diesel::postgres::Object;
use orm::migrations::run_migrations;
use shared::block::Block;
use shared::block_result::BlockResult;
use shared::checksums::Checksums;
use shared::crawler::crawl;
use shared::crawler_state::BlockCrawlerState;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use shared::id::Id;
use tendermint_rpc::HttpClient;
use transactions::app_state::AppState;
use transactions::config::AppConfig;
use transactions::repository::{
    block as block_repo, transactions as transaction_repo,
};
use transactions::services::{
    db as db_service, namada as namada_service,
    tendermint as tendermint_service, tx as tx_service,
};

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    config.log.init();

    let client =
        Arc::new(HttpClient::new(config.tendermint_url.as_str()).unwrap());

    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = namada_service::query_tx_code_hash(&client, &code_path)
            .await
            .unwrap_or_else(|| {
                panic!("{} must be defined in namada storage.", code_path)
            });
        checksums.add(code_path, code.to_lowercase());
    }

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    let crawler_state = db_service::get_crawler_state(&conn).await;

    let next_block = match config.backfill_from {
        Some(height) => {
            tracing::warn!("Backfilling from block height {}", height);
            height
        }
        None => std::cmp::max(
            crawler_state
                .map(|cs| cs.last_processed_block + 1)
                .unwrap_or(1),
            config.from_block_height,
        ),
    };

    crawl(
        move |block_height| {
            crawling_fn(
                block_height,
                client.clone(),
                conn.clone(),
                checksums.clone(),
                config.backfill_from.is_none(),
            )
        },
        next_block,
        None,
    )
    .await
}

async fn crawling_fn(
    block_height: u32,
    client: Arc<HttpClient>,
    conn: Arc<Object>,
    checksums: Checksums,
    should_update_crawler_state: bool,
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

    tracing::debug!(block = block_height, "Query block...");
    let tm_block_response =
        tendermint_service::query_raw_block_at_height(&client, block_height)
            .await
            .into_rpc_error()?;
    tracing::debug!(
        block = block_height,
        "Raw block contains {} txs...",
        tm_block_response.block.data.len()
    );

    tracing::debug!(block = block_height, "Query block results...");
    let tm_block_results_response =
        tendermint_service::query_raw_block_results_at_height(
            &client,
            block_height,
        )
        .await
        .into_rpc_error()?;
    let block_results = BlockResult::from(tm_block_results_response);

    let proposer_address_namada = namada_service::get_validator_namada_address(
        &client,
        &Id::from(&tm_block_response.block.header.proposer_address),
    )
    .await
    .into_rpc_error()?;

    tracing::debug!(
        block = block_height,
        tm_address = tm_block_response.block.header.proposer_address.to_string(),
        namada_address = ?proposer_address_namada,
        "Got block proposer address"
    );

    let block = Block::from(
        &tm_block_response,
        &block_results,
        &proposer_address_namada,
        checksums,
        1_u32, // placeholder, we dont need the epoch here
        block_height,
    );

    let inner_txs = block.inner_txs();
    let wrapper_txs = block.wrapper_txs();
    let transaction_sources = block.sources();
    let gas_estimates = tx_service::get_gas_estimates(&inner_txs, &wrapper_txs);

    let ibc_sequence_packet =
        tx_service::get_ibc_packets(&block_results, &inner_txs);
    let ibc_ack_packet = tx_service::get_ibc_ack_packet(&inner_txs);

    tracing::info!(
        "Deserialized {} wrappers, {} inners, {} ibc sequence numbers and {} \
         ibc acks events...",
        wrapper_txs.len(),
        inner_txs.len(),
        ibc_sequence_packet.len(),
        ibc_ack_packet.len()
    );

    // Because transaction crawler starts from block 1 we read timestamp from
    // the block
    let timestamp = tm_block_response.block.header.time.unix_timestamp();
    let crawler_state = BlockCrawlerState {
        timestamp,
        last_processed_block: block_height,
    };

    tracing::info!(
        wrapper_txs = wrapper_txs.len(),
        inner_txs = inner_txs.len(),
        block = block_height,
        "Queried block successfully",
    );

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                block_repo::upsert_block(
                    transaction_conn,
                    block,
                    tm_block_response,
                )?;
                transaction_repo::insert_wrapper_transactions(
                    transaction_conn,
                    wrapper_txs,
                )?;
                transaction_repo::insert_inner_transactions(
                    transaction_conn,
                    inner_txs,
                )?;

                if should_update_crawler_state {
                    transaction_repo::insert_crawler_state(
                        transaction_conn,
                        crawler_state,
                    )?;
                }

                transaction_repo::insert_ibc_sequence(
                    transaction_conn,
                    ibc_sequence_packet,
                )?;

                transaction_repo::update_ibc_sequence(
                    transaction_conn,
                    ibc_ack_packet,
                )?;

                transaction_repo::insert_transactions_history(
                    transaction_conn,
                    transaction_sources,
                )?;

                transaction_repo::insert_gas_estimates(
                    transaction_conn,
                    gas_estimates,
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()?;

    tracing::info!(block = block_height, "Inserted block into database",);

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
        transaction_repo::update_crawler_timestamp(
            transaction_conn,
            timestamp,
        )?;

        anyhow::Ok(())
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Insert crawler state error")
    .into_db_error()
}
