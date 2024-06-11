use diesel::associations::Associations;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use shared::unbond::Unbond;

use crate::schema::unbonds;
use crate::validators::ValidatorDb;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = unbonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnbondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: String,
    pub withdraw_epoch: i32,
}

#[derive(Identifiable, Clone, Queryable, Selectable, Associations)]
#[diesel(table_name = unbonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(ValidatorDb, foreign_key = validator_id))]
pub struct UnbondDb {
    pub id: i32,
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: String,
    pub withdraw_epoch: i32,
}

impl UnbondInsertDb {
    pub fn from_unbond(unbond: Unbond, validator_id: i32) -> Self {
        Self {
            address: unbond.source.to_string(),
            validator_id,
            raw_amount: unbond.amount.to_string(),
            withdraw_epoch: unbond.withdraw_at as i32,
        }
    }
}
