use std::collections::HashSet;

use anyhow::Context;
use deadpool_diesel::postgres::Object;
use diesel::dsl::not;
use diesel::upsert::excluded;
use diesel::{
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::schema::validators;
use orm::validators::{ValidatorDb, ValidatorInsertDb};
use shared::error::ContextDbInteractError;
use shared::validator::Validator;

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
        ))
        .execute(transaction_conn)
        .context("Failed to update validators in db")?;

    Ok(())
}

pub async fn get_missing_validators(
    conn: &Object,
    validators: HashSet<Validator>,
) -> anyhow::Result<Vec<ValidatorDb>> {
    conn.interact(move |conn| {
        validators::table
            .filter(not(validators::dsl::namada_address.eq_any(
                validators
                    .into_iter()
                    .map(|validator| validator.address.to_owned().to_string())
                    .collect::<Vec<_>>(),
            )))
            .select(ValidatorDb::as_select())
            .load(conn)
    })
    .await
    .context_db_interact_error()?
    .context("Failed to read validator state from the db")
}
