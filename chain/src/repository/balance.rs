use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::balances::BalancesInsertDb;
use orm::schema::balances;
use shared::balance::Balances;

pub fn insert_balance(
    transaction_conn: &mut PgConnection,
    balances: Balances,
) -> anyhow::Result<()> {
    diesel::insert_into(balances::table)
        .values::<&Vec<BalancesInsertDb>>(
            &balances
                .into_iter()
                .map(BalancesInsertDb::from_balance)
                .collect::<Vec<_>>(),
        )
        .on_conflict((balances::columns::owner, balances::columns::token))
        .do_update()
        .set(
            balances::columns::raw_amount
                .eq(excluded(balances::columns::raw_amount)),
        )
        .execute(transaction_conn)
        .context("Failed to update balances in db")?;

    anyhow::Ok(())
}
