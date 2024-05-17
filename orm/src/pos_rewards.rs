use diesel::{Insertable, Queryable, Selectable};
use shared::rewards::Reward;

use crate::schema::pos_rewards;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = pos_rewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PosRewardInsertDb {
    pub owner: String,
    pub validator_id: i32,
    pub raw_amount: String,
}

pub type PoSRewardDb = PosRewardInsertDb;

impl PosRewardInsertDb {
    pub fn from_reward(reward: Reward, validator_id: i32) -> Self {
        Self {
            owner: reward.delegation_pair.delegator_address.to_string(),
            raw_amount: reward.amount.to_string(),
            validator_id,
        }
    }
}
