use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};
use orm::schema::pos_rewards;
use shared::block::Epoch;
use shared::error::ContextDbInteractError;

pub async fn query_last_indexed_epoch(conn: Object) -> anyhow::Result<Epoch> {
    let epoch = conn
        .interact(move |conn| {
            pos_rewards::dsl::pos_rewards
                .select(max(pos_rewards::dsl::epoch))
                .first::<Option<i32>>(conn)
        })
        .await
        .context_db_interact_error()?
        .context("Failed to read epoch in table pos_rewards")?;

    Ok(epoch.map(|h| h as Epoch).unwrap_or(0 as Epoch))
}
