use diesel::associations::Associations;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use shared::bond::Bond;

use crate::schema::bonds;
use crate::validators::ValidatorDb;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = bonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BondInsertDb {
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: String,
    pub start: i32,
}

#[derive(Identifiable, Clone, Queryable, Selectable, Associations)]
#[diesel(table_name = bonds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(ValidatorDb, foreign_key = validator_id))]
pub struct BondDb {
    pub id: i32,
    pub address: String,
    pub validator_id: i32,
    pub raw_amount: String,
    pub start: i32,
}

impl BondInsertDb {
    pub fn from_bond(bond: Bond, validator_id: i32) -> Self {
        Self {
            address: bond.source.to_string(),
            validator_id,
            raw_amount: bond.amount.to_string(),
            start: bond.start as i32,
        }
    }
}
