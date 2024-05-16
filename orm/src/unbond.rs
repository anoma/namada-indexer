use diesel::data_types::PgNumeric;
use diesel::{Insertable, Queryable, Selectable};
use shared::unbond::Unbond;

use crate::schema::unbonds;
use crate::utils::{Base10000BigUint, PgNumericInt};

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = unbonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnbondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: PgNumeric,
    pub epoch: i32,
    pub withdraw_epoch: i32,
}

pub type UnbondDb = UnbondInsertDb;

impl UnbondInsertDb {
    pub fn from_unbond(unbond: Unbond, validator_id: i32) -> Self {
        let num = Base10000BigUint::from(unbond.amount);
        let raw_amount = PgNumericInt::from(num);

        Self {
            address: unbond.source.to_string(),
            validator_id,
            raw_amount: raw_amount.into_inner(),
            epoch: unbond.epoch as i32,
            withdraw_epoch: unbond.withdraw_at as i32,
        }
    }
}
