use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::tx_crawler_state;
use shared::crawler_state::CrawlerState;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = tx_crawler_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CrawlerStateDb {
    pub id: i32,
    pub height: i32,
    pub epoch: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = tx_crawler_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CrawlerStateInsertDb {
    pub height: i32,
    pub epoch: i32,
}

impl From<CrawlerState> for CrawlerStateInsertDb {
    fn from(crawler_state: CrawlerState) -> Self {
        Self {
            height: crawler_state.height as i32,
            epoch: crawler_state.epoch as i32,
        }
    }
}
