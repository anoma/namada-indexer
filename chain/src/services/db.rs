use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use orm::crawler_state::{
    BlockCrawlerStateDb, CrawlerNameDb, EpochCrawlerStateDb,
};
use orm::schema::crawler_state;
use shared::block::{BlockHeight, Epoch};
use shared::crawler_state::{BlockCrawlerState, EpochCrawlerState};
use shared::error::ContextDbInteractError;

pub async fn get_chain_crawler_state(
    conn: &Object,
) -> anyhow::Result<BlockCrawlerState> {
    let crawler_state: BlockCrawlerStateDb = conn
        .interact(move |conn| {
            crawler_state::table
                .filter(crawler_state::name.eq(CrawlerNameDb::Chain))
                .select((
                    crawler_state::dsl::last_processed_block,
                    crawler_state::dsl::last_processed_epoch,
                    crawler_state::dsl::timestamp,
                ))
                .first(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read chain crawler state from the db")?;

    Ok(BlockCrawlerState {
        last_processed_block: crawler_state.last_processed_block as BlockHeight,
        last_processed_epoch: crawler_state.last_processed_epoch as Epoch,
        timestamp: crawler_state.timestamp.and_utc().timestamp(),
    })
}
pub async fn get_pos_crawler_state(
    conn: &Object,
) -> anyhow::Result<EpochCrawlerState> {
    let crawler_state: EpochCrawlerStateDb = conn
        .interact(move |conn| {
            crawler_state::table
                .filter(crawler_state::name.eq(CrawlerNameDb::Pos))
                .select((
                    crawler_state::dsl::last_processed_epoch,
                    crawler_state::dsl::timestamp,
                ))
                .first(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block crawler state from the db")?;

    Ok(EpochCrawlerState {
        last_processed_epoch: crawler_state.last_processed_epoch as Epoch,
        timestamp: crawler_state.timestamp.and_utc().timestamp(),
    })
}
