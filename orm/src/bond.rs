use diesel::data_types::PgNumeric;
use diesel::Insertable;
use shared::bond::Bond;

use crate::schema::bonds;
use crate::utils::{Base10000BigUint, PgNumericInt};

#[derive(Insertable, Clone)]
#[diesel(table_name = bonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BondInsertDb {
    pub address: String,
    //TODO: is validator_id id of a row?
    pub validator_id: i32,
    pub raw_amount: PgNumeric,
}

impl BondInsertDb {
    pub fn from_bond(bond: Bond, validator_id: i32) -> Self {
        let num = Base10000BigUint::from(bond.amount);
        let raw_amount = PgNumericInt::from(num);
        println!("raw_amount {:?}", raw_amount.clone().into_inner());

        Self {
            address: bond.source.to_string(),
            validator_id,
            raw_amount: raw_amount.into_inner(),
        }
    }
}
