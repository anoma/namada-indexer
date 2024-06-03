use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::{revealed_pk::RevealedPkInsertDb, schema::revealed_pk};
use shared::{id::Id, public_key::PublicKey};

pub fn insert_revealed_pks(
    transaction_conn: &mut PgConnection,
    revealed_pks: Vec<(PublicKey, Id)>,
) -> anyhow::Result<()> {
    diesel::insert_into(revealed_pk::table)
        .values::<&Vec<RevealedPkInsertDb>>(
            &revealed_pks
                .into_iter()
                .map(|(pk, address)| RevealedPkInsertDb::from(pk, address))
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to update balances in db")?;

    anyhow::Ok(())
}
