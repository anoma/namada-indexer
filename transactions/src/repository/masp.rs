use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::masp::MaspInsertDb;
use orm::schema::masp_pool;
use shared::masp::MaspEntry;

pub fn insert_masp_entries(
    transaction_conn: &mut PgConnection,
    masp_entries: Vec<MaspEntry>,
) -> anyhow::Result<()> {
    diesel::insert_into(masp_pool::table)
        .values::<&Vec<MaspInsertDb>>(
            &masp_entries
                .into_iter()
                .map(MaspInsertDb::from)
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to insert masp pool entries in db")?;

    anyhow::Ok(())
}
