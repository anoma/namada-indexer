use diesel::Insertable;
use shared::block::Epoch;
use shared::rewards::Reward;

use crate::schema::pos_rewards;

#[derive(Insertable, Clone)]
#[diesel(table_name = pos_rewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PosRewardInsertDb {
    pub owner: String,
    pub epoch: i32,
    pub raw_amount: String,
}

impl PosRewardInsertDb {
    pub fn from_reward(reward: Reward, epoch: Epoch) -> Self {
        Self {
            owner: reward.delegation_pair.delegator_address.to_string(),
            epoch: epoch as i32,
            raw_amount: reward.amount.to_string(),
        }
    }
}
