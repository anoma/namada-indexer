use std::convert::identity;
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDateTime;
use clap::Parser;
use deadpool_diesel::postgres::Object;
use namada_sdk::state::EPOCH_SWITCH_BLOCKS_DELAY;
use namada_sdk::time::{DateTimeUtc, Utc};
use orm::gas::GasPriceDb;
use orm::migrations::run_migrations;
use orm::parameters::ParametersInsertDb;
use parameters::app_state::AppState;
use parameters::config::AppConfig;
use parameters::repository;
use parameters::services::{
    namada as namada_service, tendermint as tendermint_service,
};
use shared::crawler;
use shared::crawler_state::{CrawlerName, IntervalCrawlerState};
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    config.log.init();

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

        tracing::trace!(
            "Not enough time has passed since last crawl, waiting..."
        );

        return Err(MainError::NoAction);
    }

    tracing::debug!("Querying parameters...");

    let parameters = namada_service::get_parameters(&client)
        .await
        .into_rpc_error()?;

    let genesis = tendermint_service::query_genesis(&client)
        .await
        .into_rpc_error()?;

    let checksums = namada_service::query_checksums(&client).await;

    let gas_price = namada_service::get_gas_price(&client).await;

    let timestamp = DateTimeUtc::now().0.timestamp();
    let crawler_state = IntervalCrawlerState { timestamp };

    tracing::info!("Queried parameters successfully",);

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                repository::parameters::upsert_chain_parameters(
                    transaction_conn,
                    ParametersInsertDb::from((
                        parameters,
                        genesis,
                        checksums,
                        EPOCH_SWITCH_BLOCKS_DELAY,
                    )),
                )?;

                repository::parameters::upsert_gas_price(
                    transaction_conn,
                    gas_price
                        .iter()
                        .cloned()
                        .map(GasPriceDb::from)
                        .collect::<Vec<GasPriceDb>>(),
                )?;

                repository::crawler_state::upsert_crawler_state(
                    transaction_conn,
                    (CrawlerName::Parameters, crawler_state).into(),
                )?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .and_then(identity)
    .into_db_error()?;

    tracing::info!(sleep_for = sleep_for, "Inserted parameters into database");

    // Once we are done processing, we reset the instant
    *instant = Instant::now();

    Ok(())
}

fn can_process(instant: &MutexGuard<Instant>, sleep_for: u64) -> bool {
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
