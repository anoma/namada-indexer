use diesel::data_types::PgNumeric;
use diesel::{Insertable, Queryable, Selectable};
use shared::bond::Bond;

use crate::schema::bonds;
use crate::utils::{Base10000BigUint, PgNumericInt};

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = bonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: PgNumeric,
    pub epoch: i32,
}

pub type BondDb = BondInsertDb;

impl BondInsertDb {
    pub fn from_bond(bond: Bond, validator_id: i32) -> Self {
        let num = Base10000BigUint::from(bond.amount);
        let raw_amount = PgNumericInt::from(num);

        Self {
            address: bond.source.to_string(),
            validator_id,
            raw_amount: raw_amount.into_inner(),
            epoch: bond.epoch as i32,
        }
    }
}
