use anyhow::Context;
use clap::Parser;
use webserver::app::ApplicationServer;
use webserver::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    ApplicationServer::serve(config)
        .await
        .context("could not initialize application routes")?;

    Ok(())
}
