use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};

use crate::schema::token_supplies_per_epoch;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = token_supplies_per_epoch)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenSupplies {
    pub address: String,
    pub epoch: i32,
    pub total: BigDecimal,
    pub effective: Option<BigDecimal>,
}

pub type TokenSuppliesInsertDb = TokenSupplies;
