use diesel::{Queryable, Selectable};

use crate::schema::gas;
use crate::transactions::TransactionKindDb;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = gas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasDb {
    pub token: String,
    pub tx_kind: TransactionKindDb,
    pub gas_limit: i32,
}
