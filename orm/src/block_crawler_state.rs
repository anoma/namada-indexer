use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;
use shared::crawler_state::CrawlerState;

use crate::schema::block_crawler_state;

#[derive(Serialize, Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = block_crawler_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockCrawlerStateDb {
    pub id: i32,
    pub height: i32,
    pub epoch: i32,
    pub timestamp: i64,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = block_crawler_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockCrawlerStateInsertDb {
    pub height: i32,
    pub epoch: i32,
    pub timestamp: i64,
}

impl From<CrawlerState> for BlockCrawlerStateInsertDb {
    fn from(crawler_state: CrawlerState) -> Self {
        Self {
            height: crawler_state.height as i32,
            epoch: crawler_state.epoch as i32,
            timestamp: crawler_state.timestamp,
        }
    }
}
