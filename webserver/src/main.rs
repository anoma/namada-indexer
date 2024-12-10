use anyhow::Context;
use clap::Parser;
use webserver::app::ApplicationServer;
use webserver::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    config.log.init();

    ApplicationServer::serve(config)
        .await
        .context("could not initialize application routes")?;

    Ok(())
}
