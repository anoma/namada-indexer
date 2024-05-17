use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::bond::BondInsertDb;
use orm::schema::{bonds, unbonds, validators};
use orm::unbond::UnbondInsertDb;
use orm::validators::ValidatorDb;
use shared::bond::Bonds;
use shared::unbond::Unbonds;

pub fn insert_bonds(
    transaction_conn: &mut PgConnection,
    bonds: Bonds,
) -> anyhow::Result<()> {
    diesel::insert_into(bonds::table)
        .values::<&Vec<BondInsertDb>>(
            &bonds
                .into_iter()
                .map(|bond| {
                    let validator: ValidatorDb = validators::table
                        // Epoch for validators is problematic?
                        .filter(
                            validators::namada_address
                                .eq(&bond.target.to_string()),
                        )
                        .select(ValidatorDb::as_select())
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    BondInsertDb::from_bond(bond, validator.id)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((bonds::columns::validator_id, bonds::columns::address))
        .do_update()
        .set((
            bonds::columns::raw_amount.eq(excluded(bonds::columns::raw_amount)),
        ))
        .execute(transaction_conn)
        .context("Failed to update bonds in db")?;

    anyhow::Ok(())
}

pub fn insert_unbonds(
    transaction_conn: &mut PgConnection,
    unbonds: Unbonds,
) -> anyhow::Result<()> {
    diesel::insert_into(unbonds::table)
        .values::<&Vec<UnbondInsertDb>>(
            &unbonds
                .into_iter()
                .map(|unbond| {
                    let validator: ValidatorDb = validators::table
                        .filter(
                            validators::namada_address
                                .eq(&unbond.target.to_string()),
                        )
                        .select(ValidatorDb::as_select())
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    UnbondInsertDb::from_unbond(unbond, validator.id)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            unbonds::columns::validator_id,
            unbonds::columns::address,
            unbonds::columns::withdraw_epoch,
        ))
        .do_update()
        .set((
            unbonds::columns::raw_amount
                .eq(excluded(unbonds::columns::raw_amount)),
            unbonds::columns::withdraw_epoch
                .eq(excluded(unbonds::columns::withdraw_epoch)),
        ))
        .execute(transaction_conn)
        .context("Failed to update unbonds in db")?;
    anyhow::Ok(())
}
