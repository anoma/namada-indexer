use diesel::pg::Pg;
use diesel::sql_types::Nullable;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use shared::crawler_status::{
    BlockCrawlerStatus, CrawlerName, EpochCrawlerStatus, IntervalCrawlerStatus,
};

use crate::schema::crawler_status;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::CrawlerName"]
pub enum CrawlerNameDb {
    Chain,
    Governance,
    Parameters,
    Pos,
    Rewards,
    Transactions,
}

impl From<CrawlerName> for CrawlerNameDb {
    fn from(value: CrawlerName) -> Self {
        match value {
            CrawlerName::Chain => Self::Chain,
            CrawlerName::Governance => Self::Governance,
            CrawlerName::Parameters => Self::Parameters,
            CrawlerName::Pos => Self::Pos,
            CrawlerName::Rewards => Self::Rewards,
            CrawlerName::Transactions => Self::Transactions,
        }
    }
}

// TODO: rename tp state
#[derive(Serialize, Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crawler_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CrawlerStatusDb {
    pub name: CrawlerNameDb,
    pub last_processed_block: Option<i32>,
    pub last_processed_epoch: Option<i32>,
    pub timestamp: chrono::NaiveDateTime,
}

// TODO: rename tp state
#[derive(Serialize, Clone, Debug)]
pub struct BlockCrawlerStatusDb {
    pub last_processed_block: i32,
    pub last_processed_epoch: i32,
    pub timestamp: chrono::NaiveDateTime,
}
impl
    Queryable<
        (
            Nullable<diesel::sql_types::Integer>,
            Nullable<diesel::sql_types::Integer>,
            diesel::sql_types::Timestamp,
        ),
        Pg,
    > for BlockCrawlerStatusDb
{
    type Row = (Option<i32>, Option<i32>, chrono::NaiveDateTime);

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        match row {
            (
                Some(last_processed_block),
                Some(last_processed_epoch),
                timestamp,
            ) => Ok(Self {
                last_processed_block,
                last_processed_epoch,
                timestamp,
            }),
            _ => Err(
                "last_processed_block or last_processed_epoch missing in the \
                 epoch crawler status"
                    .into(),
            ),
        }
    }
}

// TODO: rename tp state
#[derive(Serialize, Clone, Debug)]
pub struct EpochCrawlerStatusDb {
    pub last_processed_epoch: i32,
    pub timestamp: chrono::NaiveDateTime,
}
impl
    Queryable<
        (
            Nullable<diesel::sql_types::Integer>,
            diesel::sql_types::Timestamp,
        ),
        Pg,
    > for EpochCrawlerStatusDb
{
    type Row = (Option<i32>, chrono::NaiveDateTime);

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        match row {
            (Some(last_processed_epoch), timestamp) => Ok(Self {
                last_processed_epoch,
                timestamp,
            }),
            _ => {
                Err("last_processed_epoch missing in the chain crawler status"
                    .into())
            }
        }
    }
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = crawler_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockStatusInsertDb {
    pub name: CrawlerNameDb,
    pub last_processed_block: i32,
    pub last_processed_epoch: i32,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = crawler_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EpochStatusInsertDb {
    pub name: CrawlerNameDb,
    pub last_processed_epoch: i32,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = crawler_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IntervalStatusInsertDb {
    pub name: CrawlerNameDb,
    pub timestamp: chrono::NaiveDateTime,
}

impl From<(CrawlerName, BlockCrawlerStatus)> for BlockStatusInsertDb {
    fn from((crawler_name, state): (CrawlerName, BlockCrawlerStatus)) -> Self {
        let timestamp = chrono::DateTime::from_timestamp(state.timestamp, 0)
            .expect("Invalid timestamp")
            .naive_utc();

        Self {
            name: crawler_name.into(),
            last_processed_block: state.last_processed_block as i32,
            last_processed_epoch: state.last_processed_epoch as i32,
            timestamp,
        }
    }
}

impl From<(CrawlerName, EpochCrawlerStatus)> for EpochStatusInsertDb {
    fn from((crawler_name, state): (CrawlerName, EpochCrawlerStatus)) -> Self {
        let timestamp = chrono::DateTime::from_timestamp(state.timestamp, 0)
            .expect("Invalid timestamp")
            .naive_utc();

        Self {
            name: crawler_name.into(),
            last_processed_epoch: state.last_processed_epoch as i32,
            timestamp,
        }
    }
}

impl From<(CrawlerName, IntervalCrawlerStatus)> for IntervalStatusInsertDb {
    fn from(
        (crawler_name, state): (CrawlerName, IntervalCrawlerStatus),
    ) -> Self {
        let timestamp = chrono::DateTime::from_timestamp(state.timestamp, 0)
            .expect("Invalid timestamp")
            .naive_utc();

        Self {
            name: crawler_name.into(),
            timestamp,
        }
    }
}
