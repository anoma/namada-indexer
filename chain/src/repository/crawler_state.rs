use anyhow::Context;
use chrono::NaiveDateTime;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::crawler_state::{BlockStateInsertDb, CrawlerNameDb};
use orm::schema::crawler_state;
use shared::crawler_state::{BlockCrawlerState, CrawlerName};

pub fn upsert_crawler_state(
    transaction_conn: &mut PgConnection,
    crawler_state: BlockCrawlerState,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_state::table)
        .values::<&BlockStateInsertDb>(
            &(CrawlerName::Chain, crawler_state).into(),
        )
        .on_conflict(crawler_state::name)
        .do_update()
        .set((
            crawler_state::timestamp.eq(excluded(crawler_state::timestamp)),
            crawler_state::last_processed_block
                .eq(excluded(crawler_state::last_processed_block)),
            crawler_state::last_processed_epoch
                .eq(excluded(crawler_state::last_processed_epoch)),
        ))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    anyhow::Ok(())
}

pub fn update_crawler_timestamp(
    transaction_conn: &mut PgConnection,
    timestamp: NaiveDateTime,
) -> anyhow::Result<()> {
    diesel::update(crawler_state::table)
        .filter(crawler_state::name.eq(CrawlerNameDb::from(CrawlerName::Chain)))
        .set(crawler_state::timestamp.eq(timestamp))
        .execute(transaction_conn)
        .context("Failed to update crawler timestamp in db")?;

    anyhow::Ok(())
}
