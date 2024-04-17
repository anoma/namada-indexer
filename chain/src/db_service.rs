use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::{dsl::max, QueryDsl, RunQueryDsl};
use orm::schema::tx_crawler_state;

use shared::block::BlockHeight;

use crate::error::ContextDbInteractError;

pub async fn get_last_synched_block(
    conn: &Object,
) -> anyhow::Result<BlockHeight> {
    let block_height = conn
        .interact(move |conn| {
            tx_crawler_state::dsl::tx_crawler_state
                .select(max(tx_crawler_state::dsl::height))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(block_height
        .map(|h| h as BlockHeight)
        .unwrap_or(0 as BlockHeight))
}
