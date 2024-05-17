use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use chain::app_state::AppState;
use chain::config::AppConfig;
use chain::repository;
use chain::services::db::get_pos_crawler_state;
use chain::services::namada::{
    query_all_balances, query_all_bonds_and_unbonds, query_all_proposals,
    query_last_block_height,
};
use chain::services::{
    db as db_service, namada as namada_service,
    tendermint as tendermint_service,
};
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use orm::schema::pos_rewards;
use shared::block::Block;
use shared::block_result::BlockResult;
use shared::checksums::Checksums;
use shared::crawler::crawl;
use shared::crawler_state::CrawlerState;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tokio::time::sleep;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    // TODO: run migrations
    let client = HttpClient::new(config.tendermint_url.as_str()).unwrap();

    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = namada_service::query_tx_code_hash(&client, &code_path)
            .await
            .unwrap_or_else(|| {
                panic!("{} must be defined in namada storage.", code_path)
            });
        checksums.add(code_path, code);
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

    initial_query(&client, &conn).await?;

    let last_block_height = db_service::get_last_synched_block(&conn)
        .await
        .into_db_error()?;

    let next_block =
        last_block_height.expect("Last block height has to be set!");

    crawl(
        move |block_height| {
            crawling_fn(
                block_height,
                client.clone(),
                conn.clone(),
                checksums.clone(),
            )
        },
        next_block,
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

    let block =
        Block::from(tm_block_response, &block_results, checksums, epoch);
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

    let proposals_votes = block.governance_votes();
    tracing::info!("Creating {} governance votes...", proposals_votes.len());

    let addresses = block.bond_addresses();
    let bonds = namada_service::query_bonds(&client, addresses, epoch)
        .await
        .into_rpc_error()?;
    tracing::info!("Updating bonds for {} addresses", bonds.len());

    let addresses = block.unbond_addresses();
    let unbonds = namada_service::query_unbonds(&client, addresses)
        .await
        .into_rpc_error()?;
    tracing::info!("Updating unbonds for {} addresses", unbonds.len());

    let reward_claimers = block.pos_rewards();

    let crawler_state = CrawlerState::new(block_height, epoch);

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_balance(
                    transaction_conn,
                    balances,
                )?;

                repository::gov::insert_proposals(transaction_conn, proposals)?;
                repository::gov::insert_votes(
                    transaction_conn,
                    proposals_votes,
                )?;

                repository::pos::insert_bonds(transaction_conn, bonds)?;
                repository::pos::insert_unbonds(transaction_conn, unbonds)?;

                repository::crawler::insert_crawler_state(
                    transaction_conn,
                    crawler_state,
                )?;

                diesel::delete(pos_rewards::table.filter(
                    pos_rewards::dsl::owner.eq_any(
                        reward_claimers.into_iter().map(|id| id.to_string()),
                    ),
                ))
                .execute(transaction_conn)?;

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

    loop {
        let pos_crawler_epoch =
            get_pos_crawler_state(conn).await.into_db_error();

        match pos_crawler_epoch {
            Ok(pos_crawler_epoch) if pos_crawler_epoch.epoch == epoch => {
                break;
            }
            _ => {}
        }

        tracing::info!("Waiting for PoS service update...");

        sleep(Duration::from_secs(5)).await;
    }

    let balances = query_all_balances(client).await.into_rpc_error()?;

    tracing::info!("Querying bonds and unbonds...");
    let (bonds, unbonds) =
        query_all_bonds_and_unbonds(client).await.into_rpc_error()?;

    tracing::info!("Querying proposals...");
    let proposals = query_all_proposals(client).await.into_rpc_error()?;

    let crawler_state = CrawlerState::new(block_height, epoch);

    tracing::info!("Inserting initial data... {:?}", balances);

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::balance::insert_balance(
                    transaction_conn,
                    balances,
                )?;

                repository::gov::insert_proposals(transaction_conn, proposals)?;

                repository::pos::insert_bonds(transaction_conn, bonds)?;
                repository::pos::insert_unbonds(transaction_conn, unbonds)?;

                repository::crawler::insert_crawler_state(
                    transaction_conn,
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
