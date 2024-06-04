use std::collections::HashSet;

use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::bond::BondInsertDb;
use orm::schema::{bonds, pos_rewards, unbonds, validators};
use orm::unbond::UnbondInsertDb;
use orm::validators::{ValidatorDb, ValidatorUpdateMetadataDb};
use shared::bond::Bonds;
use shared::id::Id;
use shared::unbond::Unbonds;
use shared::validator::ValidatorMetadataChange;

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

pub fn delete_claimed_rewards(
    transaction_conn: &mut PgConnection,
    reward_claimers: HashSet<Id>,
) -> anyhow::Result<()> {
    diesel::delete(
        pos_rewards::table.filter(
            pos_rewards::dsl::owner
                .eq_any(reward_claimers.into_iter().map(|id| id.to_string())),
        ),
    )
    .execute(transaction_conn)
    .context("Failed to update reawrds in db")?;

    anyhow::Ok(())
}

pub fn update_validator_metadata(
    transaction_conn: &mut PgConnection,
    metadata_change: Vec<ValidatorMetadataChange>,
) -> anyhow::Result<()> {
    for metadata in metadata_change {
        let metadata_change_db = ValidatorUpdateMetadataDb {
            commission: metadata.commission,
            name: metadata.name,
            email: metadata.email,
            website: metadata.website,
            description: metadata.description,
            discord_handle: metadata.discord_handler,
            avatar: metadata.avatar,
        };
        diesel::update(validators::table)
            .set(metadata_change_db)
            .filter(validators::namada_address.eq(metadata.address.to_string()))
            .execute(transaction_conn)
            .context("Failed to update unbonds in db")?;
    }
    anyhow::Ok(())
}
