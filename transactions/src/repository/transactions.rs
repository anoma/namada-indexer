use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::crawler_state::IntervalStatusInsertDb;
use orm::schema::{crawler_state, inner_transactions, wrapper_transactions};
use orm::transactions::{InnerTransactionInsertDb, WrapperTransactionInsertDb};
use shared::crawler_state::{CrawlerName, IntervalCrawlerState};
use shared::transaction::{InnerTransaction, WrapperTransaction};

pub fn insert_inner_transactions(
    transaction_conn: &mut PgConnection,
    txs: Vec<InnerTransaction>,
) -> anyhow::Result<()> {
    diesel::insert_into(inner_transactions::table)
        .values::<&Vec<InnerTransactionInsertDb>>(
            &txs.into_iter()
                .map(InnerTransactionInsertDb::from)
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to insert inner transactions in db")?;

    anyhow::Ok(())
}

pub fn insert_wrapper_transactions(
    transaction_conn: &mut PgConnection,
    txs: Vec<WrapperTransaction>,
) -> anyhow::Result<()> {
    diesel::insert_into(wrapper_transactions::table)
        .values::<&Vec<WrapperTransactionInsertDb>>(
            &txs.into_iter()
                .map(WrapperTransactionInsertDb::from)
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to insert wrapper transactions in db")?;

    anyhow::Ok(())
}

pub fn insert_crawler_state(
    transaction_conn: &mut PgConnection,
    crawler_state: IntervalCrawlerState,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_state::table)
        .values::<&IntervalStatusInsertDb>(
            &(CrawlerName::Transactions, crawler_state).into(),
        )
        .on_conflict(crawler_state::name)
        .do_update()
        .set(crawler_state::timestamp.eq(excluded(crawler_state::timestamp)))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    anyhow::Ok(())
}
