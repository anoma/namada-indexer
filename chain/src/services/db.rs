use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use orm::crawler_state::{
    ChainCrawlerStateDb, CrawlerNameDb, EpochCrawlerStateDb,
};
use orm::schema::crawler_state;
use shared::block::{BlockHeight, Epoch};
use shared::crawler_state::{ChainCrawlerState, EpochCrawlerState};
use shared::error::ContextDbInteractError;

pub async fn try_get_chain_crawler_state(
    conn: &Object,
) -> anyhow::Result<Option<ChainCrawlerState>> {
    let crawler_state: Option<ChainCrawlerStateDb> = conn
        .interact(move |conn| {
            crawler_state::table
                .filter(crawler_state::name.eq(CrawlerNameDb::Chain))
                .select((
                    crawler_state::dsl::last_processed_block,
                    crawler_state::dsl::last_processed_epoch,
                    crawler_state::dsl::first_block_in_epoch,
                    crawler_state::dsl::timestamp,
                ))
                .first(conn)
                .optional()
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read chain crawler state from the db")?;

    match crawler_state {
        Some(crawler_state) => Ok(Some(ChainCrawlerState {
            last_processed_block: crawler_state.last_processed_block
                as BlockHeight,
            last_processed_epoch: crawler_state.last_processed_epoch as Epoch,
            first_block_in_epoch: crawler_state.first_block_in_epoch
                as BlockHeight,
            timestamp: crawler_state.timestamp.and_utc().timestamp(),
        })),
        None => Ok(None),
    }
}

pub async fn get_chain_crawler_state(
    conn: &Object,
) -> anyhow::Result<ChainCrawlerState> {
    if let Some(crawler_state) = try_get_chain_crawler_state(conn).await? {
        Ok(crawler_state)
    } else {
        Err(anyhow::format_err!("Chain crawler state not found"))
            .context_db_interact_error()
    }
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
