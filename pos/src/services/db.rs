use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use orm::schema::epoch_crawler_state;
use shared::block::Epoch;
use shared::error::ContextDbInteractError;

pub async fn get_last_synched_epoch(conn: &Object) -> anyhow::Result<Epoch> {
    let epoch = conn
        .interact(move |conn| {
            epoch_crawler_state::dsl::epoch_crawler_state
                .select(max(epoch_crawler_state::dsl::epoch))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read block max height in db")?;

    Ok(epoch.map(|h| h as Epoch).unwrap_or(0 as Epoch))
}
