use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::block_crawler_state::BlockCrawlerStateInsertDb;
use orm::schema::block_crawler_state;
use shared::crawler_state::CrawlerState;

pub fn insert_crawler_state(
    transaction_conn: &mut PgConnection,
    crawler_state: CrawlerState,
) -> anyhow::Result<()> {
    diesel::insert_into(block_crawler_state::table)
        .values::<&BlockCrawlerStateInsertDb>(&crawler_state.into())
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    anyhow::Ok(())
}
