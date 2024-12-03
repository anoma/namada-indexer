use diesel::prelude::Queryable;
use diesel::{AsChangeset, Insertable, Selectable};
use serde::{Deserialize, Serialize};
use shared::transaction::{IbcAckStatus, IbcSequence};

use crate::schema::ibc_ack;

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
