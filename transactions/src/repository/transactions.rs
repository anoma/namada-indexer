use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::schema::{inner_transactions, wrapper_transactions};
use orm::transactions::{InnerTransactionInsertDb, WrapperTransactionInsertDb};
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
        .execute(transaction_conn)
        .context("Failed to insert wrapper transactions in db")?;

    anyhow::Ok(())
}
