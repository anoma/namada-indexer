use bigdecimal::BigDecimal;
use diesel::Insertable;

use crate::schema::{masp_pool, masp_pool_aggregate};

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = masp_pool)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MaspDb {
    pub id: i32,
    pub token_address: String,
    pub timestamp: chrono::NaiveDateTime,
    pub raw_amount: BigDecimal,
    pub inner_tx_id: String,
}

pub type MaspInsertDb = MaspDb;

#[derive(Debug, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::MaspPoolAggregateWindow"]
pub enum MaspPoolAggregateWindow {
    OneDay,
    SevenDay,
    OneMonth,
    Inf,
}

#[derive(Debug, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::MaspPoolAggregateKind"]
pub enum MaspPoolAggregateKind {
    Inflows,
    Outflows,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = masp_pool_aggregate)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MaspPoolDb {
    pub id: i32,
    pub token_address: String,
    pub time_window: MaspPoolAggregateWindow,
    pub kind: MaspPoolAggregateKind,
    pub total_amount: BigDecimal,
}
