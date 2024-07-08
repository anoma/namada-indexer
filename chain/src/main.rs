use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use chain::app_state::AppState;
use chain::config::AppConfig;
use chain::repository;
use chain::services::db::get_pos_crawler_state;
use chain::services::namada::{
    query_all_balances, query_all_bonds_and_unbonds, query_all_proposals,
    query_bonds, query_last_block_height,
};
use chain::services::{
    db as db_service, namada as namada_service,
    tendermint as tendermint_service,
};
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use namada_sdk::time::DateTimeUtc;
use orm::migrations::run_migrations;
use shared::block::Block;
use shared::block_result::BlockResult;
use shared::checksums::Checksums;
use shared::crawler::crawl;
use shared::crawler_state::{BlockCrawlerState, CrawlerName};
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tokio::time::sleep;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    let client = HttpClient::new(config.tendermint_url.as_str()).unwrap();

    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = namada_service::query_tx_code_hash(&client, &code_path)
            .await
            .unwrap_or_else(|| {
                panic!("{} must be defined in namada storage.", code_path)
            });
        checksums.add(code_path, code.to_lowercase());
    }

    let log_level = match config.verbosity.log_level_filter() {
        LevelFilter::Off => None,
        LevelFilter::Error => Some(Level::ERROR),
        LevelFilter::Warn => Some(Level::WARN),
        LevelFilter::Info => Some(Level::INFO),
        LevelFilter::Debug => Some(Level::DEBUG),
        LevelFilter::Trace => Some(Level::TRACE),
    };
    if let Some(log_level) = log_level {
        let subscriber =
            FmtSubscriber::builder().with_max_level(log_level).finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    let client = Arc::new(client);

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    initial_query(&client, &conn, config.initial_query_retry_time).await?;

    let crawler_state = db_service::get_chain_crawler_state(&conn)
        .await
        .into_db_error()?;

    crawl(
        move |block_height| {
            crawling_fn(
                block_height,
                client.clone(),
                conn.clone(),
                checksums.clone(),
            )
        },
        crawler_state.last_processed_block,
    )
    .await
}

async fn crawling_fn(
    block_height: u32,
    client: Arc<HttpClient>,
    conn: Arc<Object>,
    checksums: Checksums,
) -> Result<(), MainError> {
    tracing::info!("Attempting to process block: {}...", block_height);

    if !namada_service::is_block_committed(&client, block_height)
        .await
        .into_rpc_error()?
    {
        tracing::warn!("Block {} was not processed, retry...", block_height);
        return Err(MainError::RpcError);
    }

    tracing::info!("Query block...");
    let tm_block_response =
        tendermint_service::query_raw_block_at_height(&client, block_height)
            .await
            .into_rpc_error()?;
    tracing::info!(
        "Raw block contains {} txs...",
        tm_block_response.block.data.len()
    );

    tracing::info!("Query block results...");
    let tm_block_results_response =
        tendermint_service::query_raw_block_results_at_height(
            &client,
            block_height,
        )
        .await
        .into_rpc_error()?;
    let block_results = BlockResult::from(tm_block_results_response);

    tracing::info!("Query epoch...");
    let epoch =
        namada_service::get_epoch_at_block_height(&client, block_height)
            .await
            .into_rpc_error()?;

    let block = Block::from(
        tm_block_response,
        &block_results,
        checksums,
        epoch,
        block_height,
    );
    tracing::info!("Deserialized {} txs...", block.transactions.len());

    let native_token = namada_service::get_native_token(&client)
        .await
        .into_rpc_error()?;

    let addresses = block.addresses_with_balance_change(native_token);
    let balances = namada_service::query_balance(&client, &addresses)
        .await
        .into_rpc_error()?;
    tracing::info!("Updating balance for {} addresses...", addresses.len());

    let next_governance_proposal_id =
        namada_service::query_next_governance_id(&client, block_height)
            .await
            .into_rpc_error()?;

    let proposals = block.governance_proposal(next_governance_proposal_id);
    tracing::info!("Creating {} governance proposals...", proposals.len());

    let proposals_with_tally =
        namada_service::query_tallies(&client, proposals)
            .await
            .into_rpc_error()?;

    let proposals_votes = block.governance_votes();
    tracing::info!("Creating {} governance votes...", proposals_votes.len());

    let addresses = block.bond_addresses();
    let bonds = query_bonds(&client, addresses).await.into_rpc_error()?;
    tracing::info!("Updating bonds for {} addresses", bonds.len());

    let addresses = block.unbond_addresses();
    let unbonds = namada_service::query_unbonds(&client, addresses)
        .await
        .into_rpc_error()?;
    tracing::info!("Updating unbonds for {} addresses", unbonds.len());

    let withdraw_addreses = block.withdraw_addresses();

    let revealed_pks = block.revealed_pks();
    tracing::info!(
        "Updating revealed pks for {} addresses",
        revealed_pks.len()
    );

    let metadata_change = block.validator_metadata();

    let reward_claimers = block.pos_rewards();

    let timestamp_in_sec = DateTimeUtc::now().0.timestamp();

    let crawler_state = BlockCrawlerState {
        last_processed_block: block_height,
        last_processed_epoch: epoch,
        timestamp: timestamp_in_sec,
    };

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_balance(
                    transaction_conn,
                    balances,
                )?;

                repository::gov::insert_proposals(
                    transaction_conn,
                    proposals_with_tally,
                )?;
                repository::gov::insert_votes(
                    transaction_conn,
                    proposals_votes,
                )?;

                repository::pos::insert_bonds(transaction_conn, bonds)?;
                repository::pos::insert_unbonds(transaction_conn, unbonds)?;
                repository::pos::remove_withdraws(
                    transaction_conn,
                    epoch,
                    withdraw_addreses,
                )?;

                repository::pos::delete_claimed_rewards(
                    transaction_conn,
                    reward_claimers,
                )?;

                repository::pos::update_validator_metadata(
                    transaction_conn,
                    metadata_change,
                )?;

                repository::revealed_pk::insert_revealed_pks(
                    transaction_conn,
                    revealed_pks,
                )?;

                repository::crawler_state::insert_crawler_state(
                    transaction_conn,
                    CrawlerName::Chain,
                    crawler_state,
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit block db transaction error")
    .into_db_error()
}

async fn initial_query(
    client: &HttpClient,
    conn: &Object,
    initial_query_retry_time: u64,
) -> Result<(), MainError> {
    tracing::info!("Querying initial data...");
    let block_height =
        query_last_block_height(client).await.into_rpc_error()?;
    let mut epoch =
        namada_service::get_epoch_at_block_height(client, block_height)
            .await
            .into_rpc_error()?;

    loop {
        let pos_crawler_state =
            get_pos_crawler_state(conn).await.into_db_error();

        match pos_crawler_state {
            // >= in case epochs are really short
            Ok(pos_crawler_state)
                if pos_crawler_state.last_processed_epoch >= epoch =>
            {
                // We assign pos crawler epoch as epoch to process
                epoch = pos_crawler_state.last_processed_epoch;
                break;
            }
            _ => {}
        }

        tracing::info!("Waiting for PoS service update...");

        sleep(Duration::from_secs(initial_query_retry_time)).await;
    }

    let balances = query_all_balances(client).await.into_rpc_error()?;

    tracing::info!("Querying bonds and unbonds...");
    let (bonds, unbonds) = query_all_bonds_and_unbonds(client, None, None)
        .await
        .into_rpc_error()?;

    tracing::info!("Querying proposals...");
    let proposals = query_all_proposals(client).await.into_rpc_error()?;
    let proposals_with_tally =
        namada_service::query_tallies(client, proposals.clone())
            .await
            .into_rpc_error()?;

    let proposals_votes = namada_service::query_all_votes(
        client,
        proposals.iter().map(|p| p.id).collect(),
    )
    .await
    .into_rpc_error()?;

    let timestamp = DateTimeUtc::now().0.timestamp();

    let crawler_state = BlockCrawlerState {
        last_processed_block: block_height,
        last_processed_epoch: epoch,
        timestamp,
    };

    tracing::info!("Inserting initial data... ");

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_balance(
                    transaction_conn,
                    balances,
                )?;

                repository::gov::insert_proposals(
                    transaction_conn,
                    proposals_with_tally,
                )?;

                repository::gov::insert_votes(
                    transaction_conn,
                    proposals_votes,
                )?;

                repository::pos::insert_bonds(transaction_conn, bonds)?;
                repository::pos::insert_unbonds(transaction_conn, unbonds)?;

                repository::crawler_state::insert_crawler_state(
                    transaction_conn,
                    CrawlerName::Chain,
                    crawler_state,
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit initial db transaction error")
    .into_db_error()
}
