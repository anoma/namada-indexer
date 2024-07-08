use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::{QueryDsl, RunQueryDsl};
use orm::crawler_status::{BlockCrawlerStatusDb, EpochCrawlerStatusDb};
use orm::schema::crawler_status;
use shared::block::{BlockHeight, Epoch};
use shared::crawler_status::{BlockCrawlerStatus, EpochCrawlerStatus};
use shared::error::ContextDbInteractError;

pub async fn get_chain_crawler_state(
    conn: &Object,
) -> anyhow::Result<BlockCrawlerStatus> {
    let crawler_status: BlockCrawlerStatusDb = conn
        .interact(move |conn| {
            crawler_status::table
                .select((
                    crawler_status::dsl::last_processed_block,
                    crawler_status::dsl::last_processed_epoch,
                    crawler_status::dsl::timestamp,
                ))
                // TODO: replace first
                .first(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(BlockCrawlerStatus {
        last_processed_block: crawler_status.last_processed_block
            as BlockHeight,
        last_processed_epoch: crawler_status.last_processed_epoch as Epoch,
        timestamp: crawler_status.timestamp.and_utc().timestamp(),
    })
}
pub async fn get_pos_crawler_state(
    conn: &Object,
) -> anyhow::Result<EpochCrawlerStatus> {
    let crawler_status: EpochCrawlerStatusDb = conn
        .interact(move |conn| {
            crawler_status::table
                .select((
                    crawler_status::dsl::last_processed_epoch,
                    crawler_status::dsl::timestamp,
                ))
                // TODO: replace first
                .first(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(EpochCrawlerStatus {
        last_processed_epoch: crawler_status.last_processed_epoch as Epoch,
        timestamp: crawler_status.timestamp.and_utc().timestamp(),
    })
}
