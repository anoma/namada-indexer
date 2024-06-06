use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use diesel::RunQueryDsl;
use orm::migrations::run_migrations;
use orm::parameters::ParametersInsertDb;
use orm::schema::chain_parameters;
use parameters::app_state::AppState;
use parameters::config::AppConfig;
use parameters::services::namada as namada_service;
use shared::error::{AsDbError, AsRpcError, ContextDbInteractError, MainError};
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::RetryIf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let retry_strategy = FixedInterval::from_millis(5000).map(jitter);
    let exit_handle = must_exit_handle();

    loop {
        if must_exit(&exit_handle) {
            break;
        }

        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || async {
                // We always start from the current epoch
                let current_epoch =
                    namada_service::get_current_epoch(&client.clone())
                        .await
                        .into_rpc_error()?;

                tracing::info!(
                    "Attempting to process epoch: {}...",
                    current_epoch
                );

                let parameters =
                    namada_service::get_parameters(&client, current_epoch)
                        .await
                        .into_rpc_error()?;

                conn.interact(move |conn| {
                    conn.build_transaction().read_write().run(
                        |transaction_conn| {
                            diesel::insert_into(chain_parameters::table)
                                .values::<&ParametersInsertDb>(
                                    &parameters.into(),
                                )
                                .on_conflict_do_nothing()
                                .execute(transaction_conn)
                                .context(
                                    "Failed to update crawler state in db",
                                )?;

                            anyhow::Ok(())
                        },
                    )
                })
                .await
                .context_db_interact_error()
                .into_db_error()?
                .context("Commit block db transaction error")
                .into_db_error()
            },
            |_: &MainError| !must_exit(&exit_handle),
        )
        .await;
    }

    Ok(())
}

#[inline]
fn must_exit(handle: &AtomicBool) -> bool {
    handle.load(atomic::Ordering::Relaxed)
}

fn must_exit_handle() -> Arc<AtomicBool> {
    let handle = Arc::new(AtomicBool::new(false));
    let task_handle = Arc::clone(&handle);
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
        tracing::info!("Ctrl-c received");
        task_handle.store(true, atomic::Ordering::Relaxed);
    });
    handle
}
