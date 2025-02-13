use bigdecimal::BigDecimal;
use diesel::prelude::Queryable;
use diesel::{AsChangeset, Insertable, Selectable};
use serde::{Deserialize, Serialize};
use shared::transaction::{IbcAckStatus, IbcSequence};

use crate::schema::{ibc_ack, ibc_rate_limits};

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::IbcStatus"]
pub enum IbcAckStatusDb {
    Unknown,
    Timeout,
    Fail,
    Success,
}

impl From<IbcAckStatus> for IbcAckStatusDb {
    fn from(value: IbcAckStatus) -> Self {
        match value {
            IbcAckStatus::Success => Self::Success,
            IbcAckStatus::Fail => Self::Fail,
            IbcAckStatus::Timeout => Self::Timeout,
            IbcAckStatus::Unknown => Self::Unknown,
        }
    }
}

#[derive(Serialize, Queryable, Insertable, Selectable, Clone, Debug)]
#[diesel(table_name = ibc_ack)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IbcAckDb {
    pub id: String,
    pub tx_hash: String,
    pub timeout: i64,
    pub status: IbcAckStatusDb,
}

pub type IbcAckInsertDb = IbcAckDb;

impl From<IbcSequence> for IbcAckInsertDb {
    fn from(value: IbcSequence) -> Self {
        Self {
            id: value.id(),
            tx_hash: value.tx_id.to_string(),
            timeout: value.timeout as i64,
            status: IbcAckStatusDb::Unknown,
        }
    }
}

#[derive(Serialize, AsChangeset, Clone)]
#[diesel(table_name = ibc_ack)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IbcSequencekStatusUpdateDb {
    pub status: IbcAckStatusDb,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = ibc_rate_limits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IbcRateLimitsDb {
    pub id: i32,
    pub address: String,
    pub epoch: i32,
    pub throughput_limit: BigDecimal,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = ibc_rate_limits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IbcRateLimitsInsertDb {
    pub address: String,
    pub epoch: i32,
    pub throughput_limit: BigDecimal,
}
