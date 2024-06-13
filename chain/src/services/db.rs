use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::block_crawler_state::BlockCrawlerStateDb;
use orm::epoch_crawler_state::EpochCralwerStateDb;
use orm::schema::{block_crawler_state, epoch_crawler_state};
use shared::block::{BlockHeight, Epoch};
use shared::crawler_state::CrawlerState;
use shared::error::ContextDbInteractError;

pub async fn get_last_synched_block(
    conn: &Object,
) -> anyhow::Result<Option<BlockHeight>> {
    let block_height = conn
        .interact(move |conn| {
            block_crawler_state::dsl::block_crawler_state
                .select(max(block_crawler_state::dsl::height))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(block_height.map(|h| h as BlockHeight))
}

pub async fn get_crawler_state(conn: &Object) -> anyhow::Result<CrawlerState> {
    let crawler_state = conn
        .interact(move |conn| {
            let max = block_crawler_state::table
                .select(max(block_crawler_state::columns::height))
                .first::<Option<i32>>(conn)
                .expect("Failed to read block max height in db");

            block_crawler_state::table
                .filter(block_crawler_state::columns::height.eq_any(max))
                .select(BlockCrawlerStateDb::as_select())
                .first::<BlockCrawlerStateDb>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(CrawlerState::new(
        crawler_state.height as BlockHeight,
        crawler_state.epoch as Epoch,
        crawler_state.timestamp,
    ))
}

pub async fn get_pos_crawler_state(
    conn: &Object,
) -> anyhow::Result<CrawlerState> {
    let crawler_state = conn
        .interact(move |conn| {
            let epoch = epoch_crawler_state::table
                .select(max(epoch_crawler_state::columns::epoch))
                .first::<Option<i32>>(conn)
                .expect("Failed to read block max height in db");

            epoch_crawler_state::table
                .filter(epoch_crawler_state::columns::epoch.eq_any(epoch))
                .select(EpochCralwerStateDb::as_select())
                .first::<EpochCralwerStateDb>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(CrawlerState::new(
        0 as BlockHeight,
        crawler_state.epoch as Epoch,
        0,
    ))
}
