use diesel::{Insertable, Queryable, Selectable};
use shared::gas::GasPrice;

use crate::schema::{gas, gas_price};
use crate::transactions::TransactionKindDb;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = gas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasDb {
    pub token: String,
    pub tx_kind: TransactionKindDb,
    pub gas_limit: i32,
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = gas_price)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasPriceDb {
    pub token: String,
    pub amount: String,
}

impl From<GasPrice> for GasPriceDb {
    fn from(value: GasPrice) -> Self {
        Self {
            token: value.token,
            amount: value.amount,
        }
    }
}
