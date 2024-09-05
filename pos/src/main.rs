use std::convert::identity;
use std::sync::Arc;

use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use deadpool_redis::redis::{self};
use namada_sdk::time::DateTimeUtc;
use orm::crawler_state::EpochStateInsertDb;
use orm::migrations::run_migrations;
use orm::validators::ValidatorInsertDb;
use pos::app_state::AppState;
use pos::config::AppConfig;
use pos::repository::{self};
use pos::services::namada as namada_service;
use redis::AsyncCommands;
use shared::crawler;
use shared::crawler_state::{CrawlerName, EpochCrawlerState};
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use shared::event_store::{
    publish, subscribe, PosEvents, PosInitializedEventV1,
};
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
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

    let client =
        Arc::new(HttpClient::new(config.tendermint_url.as_str()).unwrap());

    let app_state = Arc::new(
        AppState::new(config.database_url, config.queue_url).into_db_error()?,
    );
    let conn = app_state.get_db_connection().await.into_db_error()?;
    let (tx, rx) = oneshot::channel::<()>();

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    let (events_tx, events_rx) = mpsc::channel::<PosEvents>(100);

    let mut redis_conn =
        app_state.get_redis_connection().await.into_db_error()?;

    let last_processed_id: String = redis_conn
        .get("pos_last_processed_id")
        .await
        .ok()
        .unwrap_or("0".to_string());

    let subscriber = tokio::spawn(subscribe(
        redis_conn,
        ("pos_last_processed_id".to_string(), last_processed_id),
        events_tx,
        rx,
    ));

    let handler = tokio::spawn(message_processor(
        events_rx,
        Arc::clone(&client),
        Arc::clone(&app_state),
    ));

    tokio::select! {
        _ = must_exit_handle() => {
            tracing::info!("Received interrupt signal, shutting down...");
            tx.send(()).unwrap();
        }
        _ = handler => {
            tracing::info!("Handler finished...");
        }
        _ = subscriber => {
            tracing::info!("Subscriber finished...");
        }
    }

    Ok(())
}

fn must_exit_handle() -> JoinHandle<()> {
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
    })
}

async fn message_processor(
    mut rx: mpsc::Receiver<PosEvents>,
    client: Arc<HttpClient>,
    app_state: Arc<AppState>,
) -> anyhow::Result<()> {
    tracing::info!("Starting message processor...");
    while let Some(event) = rx.recv().await {
        tracing::info!("Received message: {:?}", event);
        match event {
            PosEvents::ChainInitializedEventV1(_) => {
                tracing::info!("Chain is ready to process...");
                let client = Arc::clone(&client);
                let conn = Arc::new(app_state.get_db_connection().await?);
                let redis_conn = app_state.get_redis_connection().await?;

                tracing::info!("Starting crawler...");
                // We always start from the current epoch
                let next_epoch =
                    namada_service::get_current_epoch(&client.clone())
                        .await
                        .into_rpc_error()?;

                {
                    // TODO: we should wait for first crawl iteration to finish
                    publish(
                        redis_conn,
                        PosEvents::PosInitializedEventV1(PosInitializedEventV1),
                    )
                    .await?;
                }

                tokio::spawn(crawler::crawl(
                    move |epoch| {
                        crawling_fn(epoch, conn.clone(), client.clone())
                    },
                    next_epoch,
                ));
            }
            PosEvents::Test(_) => {
                tracing::info!("!!!!!Test message received...!!!!!");
            }
            _ => {}
        }
    }
    tracing::info!("Message processor finished...");

    Ok(())
}

async fn crawling_fn(
    epoch_to_process: u32,
    conn: Arc<Object>,
    client: Arc<HttpClient>,
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

    let validators_set =
        namada_service::get_validator_set_at_epoch(&client, epoch_to_process)
            .await
            .into_rpc_error()?;

    tracing::info!(
        "Processing epoch {} with {} validators...",
        epoch_to_process,
        validators_set.validators.len()
    );

    let timestamp = DateTimeUtc::now().0.timestamp();
    let crawler_state = EpochCrawlerState {
        last_processed_epoch: epoch_to_process,
        timestamp,
    };
    let crawler_state: EpochStateInsertDb =
        (CrawlerName::Pos, crawler_state).into();

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                let validators_dbo = &validators_set
                    .validators
                    .into_iter()
                    .map(ValidatorInsertDb::from_validator)
                    .collect::<Vec<_>>();

                repository::pos::upsert_validators(
                    transaction_conn,
                    validators_dbo,
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
    .and_then(identity)
    .into_db_error()
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
