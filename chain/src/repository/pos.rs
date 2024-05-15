use anyhow::Context;
use diesel::{
    upsert::excluded, BoolExpressionMethods, ExpressionMethods, PgConnection,
    QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::{
    bond::BondInsertDb,
    schema::{bonds, unbonds, validators},
    unbond::UnbondInsertDb,
    validators::ValidatorDb,
};
use shared::{bond::Bonds, unbond::Unbonds};

pub fn insert_bonds(
    transaction_conn: &mut PgConnection,
    bonds: Bonds,
) -> anyhow::Result<()> {
    tracing::info!("Updating bonds {:?}", bonds.values);
    diesel::insert_into(bonds::table)
        .values::<&Vec<BondInsertDb>>(
            &bonds
                .values
                .into_iter()
                .map(|bond| {
                    let validator: ValidatorDb = validators::table
                        // Epoch for validators is problematic?
                        .filter(
                            validators::namada_address
                                .eq(&bond.target.to_string())
                                .and(validators::epoch.eq(bonds.epoch as i32)),
                        )
                        .select(ValidatorDb::as_select())
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    BondInsertDb::from_bond(bond, validator.id, bonds.epoch)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            bonds::columns::validator_id,
            bonds::columns::address,
            bonds::columns::epoch,
        ))
        .do_update()
        .set(
            orm::schema::bonds::columns::raw_amount
                .eq(excluded(orm::schema::bonds::columns::raw_amount)),
        )
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
                .values
                .into_iter()
                .map(|unbond| {
                    let validator: ValidatorDb = validators::table
                        .filter(
                            validators::namada_address
                                .eq(&unbond.target.to_string())
                                .and(
                                    validators::epoch.eq(unbonds.epoch as i32),
                                ),
                        )
                        .select(ValidatorDb::as_select())
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    UnbondInsertDb::from_unbond(
                        unbond,
                        validator.id,
                        unbonds.epoch,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            unbonds::columns::validator_id,
            unbonds::columns::address,
            unbonds::columns::epoch,
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
