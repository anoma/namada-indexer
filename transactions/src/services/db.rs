use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use orm::schema::wrapper_transactions;
use shared::block::BlockHeight;
use shared::error::ContextDbInteractError;

pub async fn get_last_synched_block(
    conn: &Object,
) -> anyhow::Result<Option<BlockHeight>> {
    let block_height = conn
        .interact(move |conn| {
            wrapper_transactions::dsl::wrapper_transactions
                .select(max(wrapper_transactions::dsl::block_height))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context(
            "Failed to read block max height in wrapper_transaction db table",
        )?;

    Ok(block_height.map(|h| h as BlockHeight))
}
