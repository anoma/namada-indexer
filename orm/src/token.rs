use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::token::Token;

use crate::schema::token;

pub type DenominationDb = i16;

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = token)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenDb {
    pub address: String,
    pub denomination: i16,
    pub gas_price: BigDecimal,
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = token)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasPriceDb {
    #[diesel(column_name = "address")]
    pub token: String,
    #[diesel(column_name = "gas_price")]
    pub raw_amount: BigDecimal,
    pub denomination: DenominationDb,
}

impl From<Token> for TokenDb {
    fn from(value: Token) -> Self {
        Self {
            address: value.address,
            denomination: value.denomination as i16,
            gas_price: BigDecimal::from_str(&value.gas_price.to_string())
                .expect("Invalid gas_price amount"),
        }
    }
}
