use diesel::Insertable;
use shared::rewards::Reward;

use crate::schema::pos_rewards;

#[derive(Insertable, Clone)]
#[diesel(table_name = pos_rewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PosRewardInsertDb {
    pub owner: String,
    pub raw_amount: String,
}

impl PosRewardInsertDb {
    pub fn from_reward(reward: Reward) -> Self {
        Self {
            owner: reward.delegation_pair.delegator_address.to_string(),
            raw_amount: reward.amount.to_string(),
        }
    }
}
