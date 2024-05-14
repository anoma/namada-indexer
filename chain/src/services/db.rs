use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use orm::schema::block_crawler_state;
use shared::block::BlockHeight;
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
