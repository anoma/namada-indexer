use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use diesel::RunQueryDsl;
use orm::migrations::run_migrations;
use orm::parameters::ParametersInsertDb;
use orm::schema::chain_parameters;
use parameters::app_state::AppState;
use parameters::config::AppConfig;
use parameters::services::namada as namada_service;
use shared::crawler;
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

    // We always start from the current epoch
    let current_epoch = namada_service::get_current_epoch(&client.clone())
        .await
        .into_rpc_error()?;

    crawler::crawl(
        move |epoch| crawling_fn(epoch, conn.clone(), client.clone()),
        current_epoch,
    )
    .await
}

async fn crawling_fn(
    epoch_to_process: u32,
    conn: Arc<Object>,
    client: Arc<HttpClient>,
) -> Result<(), MainError> {
    tracing::info!("Attempting to process epoch: {}...", epoch_to_process);

    let parameters = namada_service::get_parameters(&client, epoch_to_process)
        .await
        .into_rpc_error()?;

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                diesel::insert_into(chain_parameters::table)
                    .values::<&ParametersInsertDb>(&parameters.into())
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
