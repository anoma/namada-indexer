use std::convert::identity;
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDateTime;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
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
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::{sleep, Instant};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

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

    tracing::info!("version: {}", env!("VERGEN_GIT_SHA").to_string());

    let client =
        Arc::new(HttpClient::new(config.tendermint_url.as_str()).unwrap());

    let app_state = AppState::new(config.database_url).into_db_error()?;

    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Initially set the instant to the current time minus the sleep_for
    // so we can start processing right away
    let instant = Arc::new(Mutex::new(
        Instant::now()
            .checked_sub(Duration::from_secs(config.sleep_for))
            .unwrap(),
    ));

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    tracing::info!("Query epoch...");
    let epoch = namada_service::get_current_epoch(&client)
        .await
        .into_rpc_error()?;

    if epoch < 2 {
        tracing::info!("Waiting for first epoch to happen...");
        sleep(Duration::from_secs(config.sleep_for)).await;
        return Ok(());
    }

    crawler::crawl(
        move |_| {
            crawling_fn(
                conn.clone(),
                client.clone(),
                instant.clone(),
                config.sleep_for,
            )
        },
        0,
    )
    .await
}

async fn crawling_fn(
    conn: Arc<Object>,
    client: Arc<HttpClient>,
    instant: Arc<Mutex<Instant>>,
    sleep_for: u64,
) -> Result<(), MainError> {
    let mut instant = instant.lock().await;

    let should_process = can_process(&instant, sleep_for);

    if !should_process {
        let timestamp = Utc::now().naive_utc();
        update_crawler_timestamp(&conn, timestamp).await?;

        tracing::warn!(
            "Not enough time has passed since last crawl, skipping..."
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

    let timestamp = DateTimeUtc::now().0.timestamp();
    let crawler_state = IntervalCrawlerState { timestamp };

    conn.interact(move |conn| {
        conn.build_transaction().read_write().run(
            |transaction_conn: &mut diesel::prelude::PgConnection| {
                let rewards_db = repository::pos_rewards::query_rewards(
                    transaction_conn,
                    rewards,
                );

                repository::pos_rewards::upsert_rewards(
                    transaction_conn,
                    rewards_db,
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

    // Once we are done processing, we reset the instant
    *instant = Instant::now();

    Ok(())
}

fn can_process(instant: &MutexGuard<Instant>, sleep_for: u64) -> bool {
    tracing::info!("Attempting to process goverance data");

    let time_elapsed = instant.elapsed().as_secs();
    time_elapsed >= sleep_for
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
