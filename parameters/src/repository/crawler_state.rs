use anyhow::Context;
use chrono::NaiveDateTime;
use diesel::{upsert::excluded, ExpressionMethods, PgConnection, RunQueryDsl};
use orm::{
    crawler_state::{CrawlerNameDb, IntervalStateInsertDb},
    schema::crawler_state,
};
use shared::crawler_state::CrawlerName;

pub fn upsert_crawler_state(
    transaction_conn: &mut PgConnection,
    status: IntervalStateInsertDb,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_state::table)
        .values::<&IntervalStateInsertDb>(&status)
        .on_conflict(crawler_state::name)
        .do_update()
        .set((crawler_state::timestamp.eq(excluded(crawler_state::timestamp)),))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    Ok(())
}

pub fn update_timestamp(
    transaction_conn: &mut PgConnection,
    timestamp: NaiveDateTime,
) -> anyhow::Result<()> {
    diesel::update(crawler_state::table)
        .filter(
            crawler_state::name
                .eq(CrawlerNameDb::from(CrawlerName::Parameters)),
        )
        .set(crawler_state::timestamp.eq(timestamp))
        .execute(transaction_conn)
        .context("Failed to update crawler timestamp in db")?;

    anyhow::Ok(())
}
