use std::convert::identity;
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDateTime;
use clap::Parser;
use deadpool_diesel::postgres::Object;
use namada_sdk::time::{DateTimeUtc, Utc};
use orm::migrations::run_migrations;
use rewards::config::AppConfig;
use rewards::repository;
use rewards::services::namada as namada_service;
use rewards::state::AppState;
use shared::crawler;
use shared::crawler_state::{CrawlerName, IntervalCrawlerState};
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    config.log.init();

    tracing::info!("version: {}", env!("VERGEN_GIT_SHA").to_string());

    let client =
        Arc::new(HttpClient::new(config.tendermint_url.as_str()).unwrap());

    let app_state = AppState::new(config.database_url).into_db_error()?;

    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    tracing::info!("Query epoch...");

    let mut epoch;
    loop {
        epoch = namada_service::get_current_epoch(&client)
            .await
            .into_rpc_error()?;

        if epoch < 2 {
            tracing::info!("Waiting for first epoch to happen...");
            sleep(Duration::from_secs(config.sleep_for)).await;
        } else {
            break;
        }
    }

    crawler::crawl(
        move |epoch| crawling_fn(conn.clone(), client.clone(), epoch),
        epoch,
        None,
    )
    .await
}

async fn crawling_fn(
    conn: Arc<Object>,
    client: Arc<HttpClient>,
    epoch_to_process: u32,
) -> Result<(), MainError> {
    let should_process = can_process(epoch_to_process, client.clone()).await?;

    if !should_process {
        let timestamp = Utc::now().naive_utc();
        update_crawler_timestamp(&conn, timestamp).await?;

        tracing::warn!(
            "Epoch {} was not processed, retry...",
            epoch_to_process
        );

        return Err(MainError::NoAction);
    }

    tracing::info!("Starting to update proposals...");

    // TODO: change this by querying all the pairs in the database
    let delegations_pairs = namada_service::query_delegation_pairs(&client)
        .await
        .into_rpc_error()?;
    let rewards = namada_service::query_rewards(&client, delegations_pairs)
        .await
        .into_rpc_error()?;
    let non_zero_rewards = rewards
        .into_iter()
        .filter(|reward| !reward.amount.is_zero())
        .collect();

    let timestamp = DateTimeUtc::now().0.timestamp();
    let crawler_state = IntervalCrawlerState { timestamp };

    conn.interact(move |conn| {
        conn.build_transaction().read_write().run(
            |transaction_conn: &mut diesel::prelude::PgConnection| {
                repository::pos_rewards::upsert_rewards(
                    transaction_conn,
                    non_zero_rewards,
                )?;

                repository::crawler_state::upsert_crawler_state(
                    transaction_conn,
                    (CrawlerName::Rewards, crawler_state).into(),
                )?;

                anyhow::Ok(())
            },
        )
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()?;

    Ok(())
}

async fn can_process(
    epoch: u32,
    client: Arc<HttpClient>,
) -> Result<bool, MainError> {
    tracing::info!("Attempting to process epoch: {}...", epoch);
    let current_epoch = namada_service::get_current_epoch(&client.clone())
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to query Namada's last committed block: {}",
                e
            );
            MainError::RpcError
        })?;

    Ok(current_epoch >= epoch)
}

async fn update_crawler_timestamp(
    conn: &Object,
    timestamp: NaiveDateTime,
) -> Result<(), MainError> {
    conn.interact(move |transaction_conn| {
        repository::crawler_state::update_timestamp(
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
