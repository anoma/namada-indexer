use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use diesel::{QueryDsl, RunQueryDsl};
use governance::config::AppConfig;
use governance::services::{db as db_service, namada as namada_service};
use governance::state::AppState;
use orm::governance_proposal::GovernanceProposalUpdateStatusDb;
use orm::schema::governance_proposals;
use shared::error::{AsDbError, AsRpcError, MainError};
use tendermint_rpc::HttpClient;
use tokio::signal;
use tokio::time::sleep;
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

    tracing::info!("version: {}", env!("VERGEN_GIT_SHA").to_string());

    let client =
        Arc::new(HttpClient::new(config.tendermint_url.as_str()).unwrap());

    let app_state = AppState::new(config.database_url).into_db_error()?;

    let retry_strategy = FixedInterval::from_millis(5000).map(jitter);
    let exit_handle = must_exit_handle();

    loop {
        if must_exit(&exit_handle) {
            break;
        }

        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || async {
                tracing::info!("Starting to update proposals...");

                let conn =
                    app_state.get_db_connection().await.into_db_error()?;

                tracing::info!("Query epoch...");
                let epoch = namada_service::query_last_epoch(&client)
                    .await
                    .into_rpc_error()?;

                let running_governance_proposals =
                    db_service::get_all_running_proposals(conn)
                        .await
                        .map_err(|_| MainError::Database)?;
                tracing::info!(
                    "Got {} proposals to be tallied...",
                    running_governance_proposals.len()
                );

                let proposals_statuses =
                    namada_service::get_governance_proposals_updates(&client, running_governance_proposals, epoch as u32)
                        .await
                        .map_err(|_| MainError::RpcError)?;
                tracing::info!("Got {} proposals statuses updates...", proposals_statuses.len());

                let conn = app_state.get_db_connection().await.into_db_error()?;
                conn.interact(move |conn| {
                    conn.build_transaction().read_write().run(
                        |transaction_conn: &mut diesel::prelude::PgConnection| {
                            for proposal_status in proposals_statuses {
                                diesel::update(governance_proposals::table.find(proposal_status.id as i32))
                                    .set::<GovernanceProposalUpdateStatusDb>(proposal_status.into())
                                    .execute(transaction_conn)?;
                            }
                            anyhow::Ok(())
                        },
                    )
                })
                .await
                .map_err(|_| MainError::Database)?
                .map_err(|_| MainError::Database)?;

                tracing::info!("Done!");

                sleep(Duration::from_secs(config.sleep_for)).await;

                Ok(())
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
