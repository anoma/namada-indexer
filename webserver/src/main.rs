use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use webserver::{app::ApplicationServer, config::AppConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    ApplicationServer::serve(config)
        .await
        .context("could not initialize application routes")?;

    Ok(())
}
