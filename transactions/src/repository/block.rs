use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::{block::BlockDb, schema::block};
use shared::block::Block;

pub fn insert_block(
    transaction_conn: &mut PgConnection,
    block: Block,
) -> anyhow::Result<()> {
    diesel::insert_into(block::table)
        .values::<BlockDb>(BlockDb::from(block))
        .execute(transaction_conn)
        .context("Failed to insert inner transactions in db")?;

    anyhow::Ok(())
}
