use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::crawler_state::BlockStateInsertDb;
use orm::schema::crawler_state;
use shared::crawler_state::{BlockCrawlerState, CrawlerName};

pub fn insert_crawler_state(
    transaction_conn: &mut PgConnection,
    crawler_name: CrawlerName,
    crawler_state: BlockCrawlerState,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_state::table)
        .values::<&BlockStateInsertDb>(&(crawler_name, crawler_state).into())
        .on_conflict(crawler_state::name)
        .do_update()
        .set((
            crawler_state::timestamp.eq(excluded(crawler_state::timestamp)),
            crawler_state::last_processed_block
                .eq(excluded(crawler_state::last_processed_block)),
        ))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    anyhow::Ok(())
}
