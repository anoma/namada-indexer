use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::balances::BalanceChangesInsertDb;
use orm::schema::balance_changes;
use shared::balance::Balances;

pub fn insert_balance(
    transaction_conn: &mut PgConnection,
    balances: Balances,
) -> anyhow::Result<()> {
    diesel::insert_into(balance_changes::table)
        .values::<&Vec<BalanceChangesInsertDb>>(
            &balances
                .into_iter()
                .map(BalanceChangesInsertDb::from_balance)
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            balance_changes::columns::owner,
            balance_changes::columns::token,
            balance_changes::columns::height,
        ))
        .do_nothing()
        .execute(transaction_conn)
        .context("Failed to update balances in db")?;

    anyhow::Ok(())
}
