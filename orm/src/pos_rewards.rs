use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use crate::schema::pos_rewards;
use shared::rewards::Reward;

#[derive(Insertable, Queryable, Selectable, Clone)]
#[diesel(table_name = pos_rewards)]
pub struct PosRewardInsertDb {
    pub owner: String,
    pub validator_id: i32,
    pub raw_amount: BigDecimal,
    pub epoch: i32,
}

pub type PoSRewardDb = PosRewardInsertDb;

impl PosRewardInsertDb {
    pub fn from_reward(reward: Reward, validator_id: i32, epoch: i32) -> Self {
        // Assuming reward.amount.as_i64() is available
        PosRewardInsertDb {
            owner: reward.delegation_pair.delegator_address.to_string(),
            validator_id,
            raw_amount: BigDecimal::from(reward.amount.as_i64()),
            epoch,
        }
    }
}
