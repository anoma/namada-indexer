use diesel::{
    upsert::excluded, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::{
    pos_rewards::PosRewardInsertDb,
    schema::{pos_rewards, validators},
    validators::ValidatorDb,
};
use shared::rewards::Reward;

pub fn query_rewards(
    transaction_conn: &mut PgConnection,
    rewards: Vec<Reward>,
) -> Vec<PosRewardInsertDb> {
    rewards
        .into_iter()
        .map(|reward| {
            let validator: ValidatorDb =
                validators::table
                    .filter(validators::dsl::namada_address.eq(
                        &reward.delegation_pair.validator_address.to_string(),
                    ))
                    .select(ValidatorDb::as_select())
                    .first(transaction_conn)
                    .expect("Failed to get validator");
            PosRewardInsertDb::from_reward(reward, validator.id)
        })
        .collect::<Vec<_>>()
}

pub fn upsert_rewards(
    transaction_conn: &mut PgConnection,
    rewards: Vec<PosRewardInsertDb>,
) -> anyhow::Result<()> {
    diesel::insert_into(pos_rewards::table)
        .values::<&Vec<PosRewardInsertDb>>(&rewards)
        .on_conflict((
            pos_rewards::columns::owner,
            pos_rewards::columns::validator_id,
        ))
        .do_update()
        .set(
            pos_rewards::columns::raw_amount
                .eq(excluded(pos_rewards::columns::raw_amount)),
        )
        .execute(transaction_conn)?;

    Ok(())
}
