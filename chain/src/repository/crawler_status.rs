use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::crawler_status::BlockStatusInsertDb;
use orm::schema::crawler_status;
use shared::crawler_status::{BlockCrawlerStatus, CrawlerName};

pub fn insert_crawler_status(
    transaction_conn: &mut PgConnection,
    crawler_name: CrawlerName,
    crawler_state: BlockCrawlerStatus,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_status::table)
        .values::<&BlockStatusInsertDb>(&(crawler_name, crawler_state).into())
        .on_conflict(crawler_status::name)
        .do_update()
        .set((
            crawler_status::timestamp.eq(excluded(crawler_status::timestamp)),
            crawler_status::last_processed_block
                .eq(excluded(crawler_status::last_processed_block)),
        ))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    anyhow::Ok(())
}
