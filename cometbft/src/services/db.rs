use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use orm::crawler_state::{
    BlockCrawlerStateDb, CometbftCrawlerStateDb, CrawlerNameDb,
};
use orm::schema::crawler_state;
use shared::block::BlockHeight;
use shared::crawler_state::{BlockCrawlerState, CometbftCrawlerState};
use shared::error::ContextDbInteractError;

pub async fn get_cometbft_crawler_state(
    conn: &Object,
) -> anyhow::Result<CometbftCrawlerState> {
    let crawler_state: CometbftCrawlerStateDb = conn
        .interact(move |conn| {
            crawler_state::table
                .filter(crawler_state::name.eq(CrawlerNameDb::Cometbft))
                .select((
                    crawler_state::dsl::last_processed_block,
                    crawler_state::dsl::timestamp,
                ))
                .first(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read chain crawler state from the db")?;

    Ok(CometbftCrawlerState {
        last_processed_block: crawler_state.last_processed_block as BlockHeight,
        timestamp: crawler_state.timestamp.and_utc().timestamp(),
    })
}

async fn get_crawler_state(
    conn: &Object,
    crawler_name: CrawlerNameDb,
) -> anyhow::Result<BlockCrawlerState> {
    let crawler_state: BlockCrawlerStateDb = conn
        .interact(move |conn| {
            crawler_state::table
                .filter(crawler_state::name.eq(crawler_name))
                .select((
                    crawler_state::dsl::last_processed_block,
                    crawler_state::dsl::timestamp,
                ))
                .first(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read chain crawler state from the db")?;

    Ok(BlockCrawlerState {
        last_processed_block: crawler_state.last_processed_block as BlockHeight,
        timestamp: crawler_state.timestamp.and_utc().timestamp(),
    })
}

pub async fn get_chain_crawler_state(
    conn: &Object,
) -> anyhow::Result<BlockCrawlerState> {
    get_crawler_state(conn, CrawlerNameDb::Chain).await
}

pub async fn get_transaction_crawler_state(
    conn: &Object,
) -> anyhow::Result<BlockCrawlerState> {
    get_crawler_state(conn, CrawlerNameDb::Transactions).await
}
