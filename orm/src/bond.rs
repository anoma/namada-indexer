use diesel::{Insertable, Queryable, Selectable};
use shared::bond::Bond;

use crate::schema::bonds;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = bonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: String,
    pub epoch: i32,
}

pub type BondDb = BondInsertDb;

impl BondInsertDb {
    pub fn from_bond(bond: Bond, validator_id: i32) -> Self {
        Self {
            address: bond.source.to_string(),
            validator_id,
            raw_amount: bond.amount.to_string(),
            epoch: bond.epoch as i32,
        }
    }
}
