use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::gas::GasPrice;

use crate::schema::{gas, gas_price};
use crate::transactions::TransactionKindDb;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = gas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasDb {
    pub tx_kind: TransactionKindDb,
    pub gas_limit: i32,
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = gas_price)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasPriceDb {
    pub token: String,
    pub amount: BigDecimal,
}

impl From<GasPrice> for GasPriceDb {
    fn from(value: GasPrice) -> Self {
        Self {
            token: value.token,
            amount: BigDecimal::from_str(&value.amount.to_string())
                .expect("Invalid amount"),
        }
    }
}
