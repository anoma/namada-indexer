use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use orm::schema::crawler_state;
use shared::block::Epoch;
use shared::error::ContextDbInteractError;

pub async fn get_last_synched_epoch(
    conn: &Object,
) -> anyhow::Result<Option<Epoch>> {
    let epoch = conn
        .interact(move |conn| {
            crawler_state::dsl::crawler_state
                .select(max(crawler_state::dsl::last_processed_epoch))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(epoch.map(|h| h as Epoch))
}
