use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use diesel::RunQueryDsl;
use orm::epoch_crawler_state::EpochCralwerStateInsertDb;
use orm::schema::{epoch_crawler_state, validators};
use orm::validators::ValidatorInsertDb;
use pos::app_state::AppState;
use pos::config::AppConfig;
use pos::services::{db as db_service, namada as namada_service};
use shared::crawler;
use shared::crawler_state::CrawlerState;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    // TODO: run migrations

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

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);
    let last_epoch = db_service::get_last_synched_epoch(&conn)
        .await
        .into_db_error()?;

    // If last processed epoch is not stored in the db, start from the current epoch
    let next_epoch = match last_epoch {
        Some(height) => height + 1,
        None => namada_service::get_current_epoch(&client.clone())
            .await
            .into_rpc_error()?,
    };

    crawler::crawl(
        move |epoch| crawling_fn(epoch, conn.clone(), client.clone()),
        next_epoch,
    )
    .await
}

async fn crawling_fn(
    epoch_to_process: u32,
    conn: Arc<Object>,
    client: Arc<HttpClient>,
) -> Result<(), MainError> {
    tracing::info!("Attempting to process epoch: {}...", epoch_to_process);
    let current_epoch = namada_service::get_current_epoch(&client.clone())
        .await
        .into_rpc_error()?;

    if current_epoch < epoch_to_process {
        tracing::warn!(
            "Epoch {} was not processed, retry...",
            epoch_to_process
        );
        return Err(MainError::RpcError);
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
    // TODO: either add height to epoch crawler or add another state struct
    let crawler_state = CrawlerState::new(0, epoch_to_process);

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                // TODO: move closure block to a function

                let validators_dbo = &validators_set
                    .validators
                    .into_iter()
                    .map(|v| {
                        ValidatorInsertDb::from_validator(v, epoch_to_process)
                    })
                    .collect::<Vec<_>>();

                diesel::insert_into(validators::table)
                    .values::<&Vec<ValidatorInsertDb>>(validators_dbo)
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update validators in db")?;

                // TODO: should we always override the db?
                diesel::insert_into(epoch_crawler_state::table)
                    .values::<&EpochCralwerStateInsertDb>(&crawler_state.into())
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit block db transaction error")
    .into_db_error()
}
