use anyhow::Context;
use clap::Parser;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Config, Runtime};
use shared::event_store::{subscribe, PosEvents};
use tokio::sync::{mpsc, oneshot};
use webserver::app::ApplicationServer;
use webserver::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let (tx, rx) = oneshot::channel();
    let (events_tx, events_rx) = mpsc::channel::<PosEvents>(100);
    let cfg = Config::from_url("redis://redis@0.0.0.0:6379");
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    let mut redis_conn = redis_pool.get().await.unwrap();

    let last_processed_id: String = redis_conn
        .get("webserver_last_processed_id")
        .await
        .ok()
        .unwrap_or("0".to_string());

    let var_name =
        ("webserver_last_processed_id".to_string(), last_processed_id);
    let subscriber =
        tokio::spawn(subscribe(redis_conn, var_name, events_tx, rx));

    let server = ApplicationServer::serve(config, events_rx);

    tokio::select! {
        _ = server => {
            tracing::info!("Received interrupt signal, shutting down...");
            tx.send(()).unwrap();
        }
        _ = subscriber => {
            tracing::info!("Subscriber finished...");
        }
    }

    Ok(())
}
