use diesel::data_types::PgNumeric;
use diesel::Insertable;
use shared::unbond::Unbond;

use crate::schema::unbonds;
use crate::utils::{Base10000BigUint, PgNumericInt};

#[derive(Insertable, Clone)]
#[diesel(table_name = unbonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnbondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: PgNumeric,
    pub withdraw_epoch: i32,
}

impl UnbondInsertDb {
    pub fn from_unbond(unbond: Unbond, validator_id: i32) -> Self {
        let num = Base10000BigUint::from(unbond.amount);
        let raw_amount = PgNumericInt::from(num);

        Self {
            address: unbond.source.to_string(),
            validator_id,
            raw_amount: raw_amount.into_inner(),
            withdraw_epoch: unbond.withdraw_at as i32,
        }
    }
}
