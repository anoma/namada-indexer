use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::balance::TokenSupply as SharedTokenSupply;

use crate::schema::token_supplies_per_epoch;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = token_supplies_per_epoch)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenSuppliesDb {
    pub id: i32,
    pub address: String,
    pub epoch: i32,
    pub total: BigDecimal,
    pub effective: Option<BigDecimal>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = token_supplies_per_epoch)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenSuppliesInsertDb {
    pub address: String,
    pub epoch: i32,
    pub total: BigDecimal,
    pub effective: Option<BigDecimal>,
}

impl From<SharedTokenSupply> for TokenSuppliesInsertDb {
    fn from(supply: SharedTokenSupply) -> Self {
        let SharedTokenSupply {
            address,
            epoch,
            total,
            effective,
        } = supply;

        Self {
            address,
            epoch,
            total,
            effective,
        }
    }
}
