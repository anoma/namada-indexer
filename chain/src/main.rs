use std::{fs::File, io::BufReader, sync::Arc};

use anyhow::Context;
use chain::{
    app_state::AppState,
    config::AppConfig,
    crawler::crawl,
    db_service::get_last_synched_block,
    error::{AsDbError, AsRpcError, ContextDbInteractError, MainError},
    namada_service, tendermint_service,
};
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use deadpool_diesel::postgres::Object;
use diesel::RunQueryDsl;
use orm::schema::tx_crawler_state;
use shared::{
    block::{deserialize_transactions, Block},
    checksums::Checksums,
    crawler_state::CrawlerState,
};
use tendermint_rpc::HttpClient;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let config = AppConfig::parse();

    tracing::info!(
        "Initializing with checksums from {:?}",
        config.checksums_filepath
    );
    let file = File::open(config.checksums_filepath).unwrap();
    let reader = BufReader::new(file);

    let mut checksums: Checksums = serde_json::from_reader(reader).unwrap();
    checksums.init();

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
    let last_block_height =
        get_last_synched_block(&conn).await.into_db_error()?;

    crawl(
        move |block_height| {
            crawling_fn(
                block_height,
                client.clone(),
                conn.clone(),
                checksums.clone(),
            )
        },
        last_block_height,
    )
    .await
}

async fn crawling_fn(
    block_height: u32,
    client: Arc<HttpClient>,
    conn: Arc<Object>,
    checksums: Checksums,
) -> Result<(), MainError> {
    tracing::info!("Attempting to process block: {}...", block_height);

    if !namada_service::is_block_committed(&client.clone(), block_height)
        .await
        .into_rpc_error()?
    {
        tracing::warn!("Block {} was not processed, retry...", block_height);
        return Err(MainError::RpcError);
    }

    tracing::info!("Query block...");
    let tm_block_response =
        tendermint_service::query_raw_block_at_height(&client, block_height)
            .await
            .into_rpc_error()?;
    tracing::info!(
        "Raw block contains {} txs...",
        tm_block_response.block.data.len()
    );

    // TODO: add later
    // tracing::info!("Query block results...");
    // let _tm_block_results_response =
    //     tendermint_service::query_raw_block_results_at_height(
    //         &client,
    //         block_height,
    //     )
    //     .await
    //     .into_rpc_error()?;

    tracing::info!("Query epoch...");
    let epoch =
        namada_service::get_epoch_at_block_height(&client, block_height)
            .await
            .into_rpc_error()?;

    let block = Block::from(tm_block_response, checksums, epoch);

    tracing::info!("Deserialized {} txs...", block.transactions.len());

    let crawler_state = CrawlerState::new(block_height, epoch);

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                diesel::insert_into(tx_crawler_state::table)
                    .values(&crawler_state.to_crawler_state_db())
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to update crawler state in db")?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    //TODO: fix later
    .unwrap()
    .context("Commit block db transaction error")
    .into_db_error()
}
