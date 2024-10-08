use std::convert::identity;
use std::sync::Arc;

use anyhow::Context;
use chain::app_state::AppState;
use chain::config::AppConfig;
use chain::repository;
use chain::services::namada::{
    query_all_balances, query_all_bonds_and_unbonds, query_all_proposals,
    query_bonds, query_last_block_height, query_tokens,
};
use chain::services::{
    db as db_service, namada as namada_service,
    tendermint as tendermint_service,
};
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use namada_sdk::time::DateTimeUtc;
use orm::migrations::run_migrations;
use shared::block::Block;
use shared::block_result::BlockResult;
use shared::checksums::Checksums;
use shared::crawler::crawl;
use shared::crawler_state::ChainCrawlerState;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use shared::id::Id;
use shared::token::Token;
use shared::validator::ValidatorSet;
use tendermint_rpc::HttpClient;
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

    initial_query(&client, &conn).await?;

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
        Some(1000),
    )
    .await
}

async fn crawling_fn(
    block_height: u32,
    client: Arc<HttpClient>,
    conn: Arc<Object>,
    checksums: Checksums,
) -> Result<(), MainError> {
    let should_process = can_process(block_height, client.clone()).await?;

    if !should_process {
        let timestamp = Utc::now().naive_utc();
        update_crawler_timestamp(&conn, timestamp).await?;

        tracing::warn!("Block {} was not processed, retry...", block_height);

        return Err(MainError::NoAction);
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

    tracing::info!("Query first block in epoch...");
    let first_block_in_epoch =
        namada_service::get_first_block_in_epoch(&client)
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

    let ibc_tokens = block.ibc_tokens().into_iter().map(Token::Ibc).collect();

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

    let validators = block.validators();
    let validator_set = ValidatorSet {
        validators: validators.clone(),
        epoch,
    };

    let addresses = block.bond_addresses();
    let bonds = query_bonds(&client, addresses).await.into_rpc_error()?;
    tracing::info!("Updating bonds for {} addresses", bonds.len());

    let bonds_updates = bonds
        .iter()
        .cloned()
        .filter_map(|(_, _, bond)| bond)
        .collect::<Vec<_>>();

    let removed_bonds_addresses = bonds
        .iter()
        .cloned()
        .map(|(source, validator, _)| (source, validator))
        .collect::<Vec<(Id, Id)>>();

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

    let crawler_state = ChainCrawlerState {
        last_processed_block: block_height,
        last_processed_epoch: epoch,
        first_block_in_epoch,
        timestamp: timestamp_in_sec,
    };

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_tokens(
                    transaction_conn,
                    ibc_tokens,
                )?;

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

                repository::pos::upsert_validators(
                    transaction_conn,
                    validator_set,
                )?;

                // We first remove all the bonds and then insert the new ones
                repository::pos::clear_bonds(
                    transaction_conn,
                    removed_bonds_addresses,
                )?;
                repository::pos::insert_bonds(transaction_conn, bonds_updates)?;

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

                repository::crawler_state::upsert_crawler_state(
                    transaction_conn,
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
) -> Result<(), MainError> {
    tracing::info!("Querying initial data...");
    let block_height =
        query_last_block_height(client).await.into_rpc_error()?;
    let epoch = namada_service::get_epoch_at_block_height(client, block_height)
        .await
        .into_rpc_error()?;
    let first_block_in_epoch = namada_service::get_first_block_in_epoch(client)
        .await
        .into_rpc_error()?;

    let tokens = query_tokens(client).await.into_rpc_error()?;

    let balances = query_all_balances(client).await.into_rpc_error()?;

    tracing::info!("Querying validators set...");
    let pipeline_length = namada_service::query_pipeline_length(client)
        .await
        .into_rpc_error()?;
    // We need to add pipeline_length to the epoch as it is possible to bond in
    // advance
    let validator_set = namada_service::get_validator_set_at_epoch(
        client,
        epoch + pipeline_length as u32,
    )
    .await
    .into_rpc_error()?;

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

    let crawler_state = ChainCrawlerState {
        last_processed_block: block_height,
        last_processed_epoch: epoch,
        first_block_in_epoch,
        timestamp,
    };

    tracing::info!("Inserting initial data... ");

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_tokens(transaction_conn, tokens)?;

                repository::balance::insert_balance_in_chunks(
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

                repository::pos::upsert_validators(
                    transaction_conn,
                    validator_set,
                )?;

                repository::pos::insert_bonds(transaction_conn, bonds)?;
                repository::pos::insert_unbonds(transaction_conn, unbonds)?;

                repository::crawler_state::upsert_crawler_state(
                    transaction_conn,
                    crawler_state,
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()
}

async fn can_process(
    block_height: u32,
    client: Arc<HttpClient>,
) -> Result<bool, MainError> {
    tracing::info!("Attempting to process block: {}...", block_height);

    let last_block_height = namada_service::query_last_block_height(&client)
        .await
        .map_err(|e| {
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
        repository::crawler_state::update_crawler_timestamp(
            transaction_conn,
            timestamp,
        )?;

        anyhow::Ok(())
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()
}
