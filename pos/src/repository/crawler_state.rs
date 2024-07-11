use anyhow::Context;
use diesel::{upsert::excluded, ExpressionMethods, PgConnection, RunQueryDsl};
use orm::{
    crawler_state::{CrawlerNameDb, EpochStateInsertDb},
    schema::crawler_state,
};
use shared::crawler_state::CrawlerName;

pub fn update_timestamp(
    transaction_conn: &mut PgConnection,
    timestamp: chrono::NaiveDateTime,
) -> anyhow::Result<()> {
    diesel::update(crawler_state::table)
        .filter(crawler_state::name.eq(CrawlerNameDb::from(CrawlerName::Pos)))
        .set(crawler_state::timestamp.eq(timestamp))
        .execute(transaction_conn)
        .context("Failed to update crawler timestamp in db")?;

    Ok(())
}

pub fn upsert_crawler_state(
    transaction_conn: &mut PgConnection,
    status: EpochStateInsertDb,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_state::table)
        .values::<&EpochStateInsertDb>(&status)
        .on_conflict(crawler_state::name)
        .do_update()
        .set((
            crawler_state::timestamp.eq(excluded(crawler_state::timestamp)),
            crawler_state::last_processed_epoch
                .eq(excluded(crawler_state::last_processed_epoch)),
        ))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    Ok(())
}
