use std::convert::identity;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use deadpool_diesel::postgres::Object;
use governance::config::AppConfig;
use governance::repository;
use governance::services::namada as namada_service;
use governance::state::AppState;
use namada_sdk::time::DateTimeUtc;
use orm::migrations::run_migrations;
use shared::crawler_state::{CrawlerName, IntervalCrawlerState};
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use shared::{client, crawler};
use tendermint_rpc::HttpClient;
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    config.log.init();

    tracing::info!("version: {}", env!("VERGEN_GIT_SHA").to_string());

    let client = Arc::new(client::build_client(&config.tendermint_url));

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
        None,
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

    tracing::info!("Query epoch...");
    let epoch = namada_service::query_last_epoch(&client)
        .await
        .into_rpc_error()?;

    let running_governance_proposals = conn
        .interact(move |conn| {
            repository::governance::get_all_running_proposals(conn)
        })
        .await
        .context_db_interact_error()
        .and_then(identity)
        .into_db_error()?;

    tracing::info!(
        "Got {} proposals to be tallied...",
        running_governance_proposals.len()
    );

    let proposals_statuses = namada_service::get_governance_proposals_updates(
        &client,
        running_governance_proposals,
        epoch as u32,
    )
    .await
    .map_err(|_| MainError::RpcError)?;
    tracing::info!(
        "Got {} proposals statuses updates...",
        proposals_statuses.len()
    );

    let timestamp = DateTimeUtc::now().0.timestamp();
    let crawler_state = IntervalCrawlerState { timestamp };

    conn.interact(move |conn| {
        conn.build_transaction().read_write().run(
            |transaction_conn: &mut diesel::prelude::PgConnection| {
                for proposal_status in proposals_statuses {
                    repository::governance::update_proposal_status(
                        transaction_conn,
                        proposal_status.id,
                        proposal_status.into(),
                    )?;
                }

                repository::crawler_state::upsert_crawler_state(
                    transaction_conn,
                    (CrawlerName::Governance, crawler_state).into(),
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
