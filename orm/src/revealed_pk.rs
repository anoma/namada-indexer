use diesel::{Insertable, Queryable, Selectable};
use shared::{id::Id, public_key::PublicKey};

use crate::schema::revealed_pk;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = revealed_pk)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RevealedPkInsertDb {
    pub pk: String,
    pub address: String,
}

pub type RevealedPkDb = RevealedPkInsertDb;

impl RevealedPkInsertDb {
    pub fn from(pk: PublicKey, address: Id) -> Self {
        Self {
            pk: pk.0,
            address: address.to_string(),
        }
    }
}
