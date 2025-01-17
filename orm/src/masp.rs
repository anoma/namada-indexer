use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::Insertable;
use shared::masp::{MaspEntry, MaspEntryDirection};

use crate::schema::{masp_pool, masp_pool_aggregate};

#[derive(Debug, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::MaspPoolDirection"]
pub enum MaspPoolDirectionDb {
    In,
    Out,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = masp_pool)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MaspDb {
    pub token_address: String,
    pub timestamp: chrono::NaiveDateTime,
    pub raw_amount: BigDecimal,
    pub direction: MaspPoolDirectionDb,
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

impl From<MaspEntry> for MaspInsertDb {
    fn from(value: MaspEntry) -> Self {
        let timestamp = chrono::DateTime::from_timestamp(value.timestamp, 0)
            .expect("Invalid timestamp")
            .naive_utc();

        let amount = BigDecimal::from_str(&value.raw_amount.to_string())
            .expect("Invalid amount");

        MaspInsertDb {
            token_address: value.token_address,
            timestamp,
            raw_amount: amount,
            direction: match value.direction {
                MaspEntryDirection::In => MaspPoolDirectionDb::In,
                MaspEntryDirection::Out => MaspPoolDirectionDb::Out,
            },
            inner_tx_id: value.inner_tx_id.to_string(),
        }
    }
}
