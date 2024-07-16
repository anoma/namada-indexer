use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::schema::validators;
use orm::validators::ValidatorInsertDb;

pub fn upsert_validators(
    transaction_conn: &mut PgConnection,
    validators_db: &Vec<ValidatorInsertDb>,
) -> anyhow::Result<()> {
    diesel::insert_into(validators::table)
        .values::<&Vec<ValidatorInsertDb>>(validators_db)
        .on_conflict(validators::columns::namada_address)
        .do_update()
        .set((
            validators::columns::voting_power
                .eq(excluded(validators::columns::voting_power)),
            validators::columns::max_commission
                .eq(excluded(validators::columns::max_commission)),
            validators::columns::commission
                .eq(excluded(validators::columns::commission)),
            validators::columns::email.eq(excluded(validators::columns::email)),
            validators::columns::website
                .eq(excluded(validators::columns::website)),
            validators::columns::description
                .eq(excluded(validators::columns::description)),
            validators::columns::discord_handle
                .eq(excluded(validators::columns::discord_handle)),
            validators::columns::avatar
                .eq(excluded(validators::columns::avatar)),
            validators::columns::state.eq(excluded(validators::columns::state)),
        ))
        .execute(transaction_conn)
        .context("Failed to update validators in db")?;

    Ok(())
}
