use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::pos_rewards::PosRewardInsertDb;
use orm::validators::ValidatorDb;
use rewards::config::AppConfig;
use rewards::services::namada as namada_service;
use rewards::state::AppState;
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

                tracing::info!("Query epoch...");
                let epoch = namada_service::get_current_epoch(&client)
                    .await
                    .into_rpc_error()?;

                if epoch < 2 {
                    tracing::info!("Waiting for first epoch to happen...");
                    sleep(Duration::from_secs(config.sleep_for)).await;
                    return Ok(());
                }

                // TODO: change this by querying all the pairs in the database
                let delegations_pairs =
                    namada_service::query_delegation_pairs(&client)
                        .await
                        .into_rpc_error()?;
                let rewards =
                    namada_service::query_rewards(&client, delegations_pairs)
                        .await
                        .into_rpc_error()?;

                let conn = app_state.get_db_connection().await.into_db_error()?;
                conn.interact(move |conn| {
                    conn.build_transaction().read_write().run(
                        |transaction_conn: &mut diesel::prelude::PgConnection| {
                            diesel::insert_into(orm::schema::pos_rewards::table)
                                .values::<&Vec<PosRewardInsertDb>>(
                                    &rewards
                                        .into_iter()
                                        .map(|reward, | {
                                            let validator: ValidatorDb = orm::schema::validators::table
                                                .filter(
                                                    orm::schema::validators::dsl::namada_address
                                                        .eq(&reward.delegation_pair.validator_address.to_string()),
                                                )
                                                .select(ValidatorDb::as_select())
                                                .first(transaction_conn)
                                                .expect("Failed to get validator");                    
                                            PosRewardInsertDb::from_reward(reward, validator.id)
                                        })
                                        .collect::<Vec<_>>(),
                                )
                                .on_conflict_do_nothing()
                                .execute(transaction_conn)
                        }
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
