use diesel::data_types::PgNumeric;
use diesel::Insertable;
use shared::balance::Balance;
use shared::block::{BlockHeight, Epoch};
use shared::rewards::Reward;

use crate::schema::pos_rewards;
use crate::utils::{Base10000BigUint, PgNumericInt};

#[derive(Insertable, Clone)]
#[diesel(table_name = pos_rewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PosRewardInsertDb {
    pub owner: String,
    pub epoch: i32,
    pub raw_amount: PgNumeric,
}

impl PosRewardInsertDb {
    pub fn from_reward(reward: Reward, epoch: Epoch) -> Self {
        let num = Base10000BigUint::from(reward.amount);
        let raw_amount = PgNumericInt::from(num);

        Self {
            owner: reward.delegation_pair.delegator_address.to_string(),
            epoch: epoch as i32,
            raw_amount: raw_amount.into_inner(),
        }
    }
}
