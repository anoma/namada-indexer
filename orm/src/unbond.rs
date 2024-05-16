use diesel::{Insertable, Queryable, Selectable};
use shared::unbond::Unbond;

use crate::schema::unbonds;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = unbonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnbondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: String,
    pub epoch: i32,
    pub withdraw_epoch: i32,
}

pub type UnbondDb = UnbondInsertDb;

impl UnbondInsertDb {
    pub fn from_unbond(unbond: Unbond, validator_id: i32) -> Self {
        Self {
            address: unbond.source.to_string(),
            validator_id,
            raw_amount: unbond.amount.to_string(),
            epoch: unbond.epoch as i32,
            withdraw_epoch: unbond.withdraw_at as i32,
        }
    }
}
