use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use orm::pos_rewards::PosRewardInsertDb;
use orm::schema::{pos_rewards, validators};
use shared::rewards::Reward;

pub fn upsert_rewards(
    transaction_conn: &mut PgConnection,
    rewards: Vec<Reward>,
) -> anyhow::Result<()> {
    diesel::insert_into(pos_rewards::table)
        .values::<Vec<PosRewardInsertDb>>(
            rewards
                .into_iter()
                .map(|reward| {
                    let validator_id: i32 = validators::table
                        .filter(
                            validators::namada_address.eq(&reward
                                .delegation_pair
                                .validator_address
                                .to_string()),
                        )
                        .select(validators::id)
                        .first(transaction_conn)
                        .expect("Failed to get validator");

                    PosRewardInsertDb::from_reward(reward, validator_id)
                })
                .collect::<Vec<_>>(),
        )
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
