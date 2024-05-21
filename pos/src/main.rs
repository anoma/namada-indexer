use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, RunQueryDsl};
use orm::epoch_crawler_state::EpochCralwerStateInsertDb;
use orm::migrations::run_migrations;
use orm::schema::{epoch_crawler_state, validators};
use orm::validators::ValidatorInsertDb;
use pos::app_state::AppState;
use pos::config::AppConfig;
use pos::repository::clear_db;
use pos::services::namada as namada_service;
use shared::crawler;
use shared::crawler_state::CrawlerState;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
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

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = Arc::new(app_state.get_db_connection().await.into_db_error()?);

    // Run migrations
    run_migrations(&conn)
        .await
        .context_db_interact_error()
        .into_db_error()?;

    // Clear db
    conn.interact(|transaction_conn| {
        clear_db(transaction_conn).into_db_error()
    })
    .await
    .context_db_interact_error()
    .into_db_error()??;

    // We always start from the current epoch
    let next_epoch = namada_service::get_current_epoch(&client.clone())
        .await
        .into_rpc_error()?;

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
    let crawler_state = CrawlerState::new(0, epoch_to_process);

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                let validators_dbo = &validators_set
                    .validators
                    .into_iter()
                    .map(ValidatorInsertDb::from_validator)
                    .collect::<Vec<_>>();

                diesel::insert_into(validators::table)
                    .values::<&Vec<ValidatorInsertDb>>(validators_dbo)
                    .on_conflict(validators::columns::namada_address)
                    .do_update()
                    .set((
                        validators::columns::voting_power
                            .eq(excluded(validators::columns::voting_power)),
                        validators::columns::max_commission
                            .eq(excluded(validators::columns::max_commission)),
                        validators::columns::commission
                            .eq(excluded(validators::columns::commission)),
                        // TODO: maybe metadata can change more often?
                        validators::columns::email
                            .eq(excluded(validators::columns::email)),
                        validators::columns::website
                            .eq(excluded(validators::columns::website)),
                        validators::columns::description
                            .eq(excluded(validators::columns::description)),
                        validators::columns::discord_handle
                            .eq(excluded(validators::columns::discord_handle)),
                        validators::columns::avatar
                            .eq(excluded(validators::columns::avatar)),
                    ))
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
