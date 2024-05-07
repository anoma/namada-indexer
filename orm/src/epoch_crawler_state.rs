use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::epoch_crawler_state;
use shared::crawler_state::CrawlerState;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = epoch_crawler_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EpochCralwerStateDb {
    pub id: i32,
    pub epoch: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = epoch_crawler_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EpochCralwerStateInsertDb {
    pub epoch: i32,
}

impl From<CrawlerState> for EpochCralwerStateInsertDb {
    fn from(crawler_state: CrawlerState) -> Self {
        Self {
            epoch: crawler_state.epoch as i32,
        }
    }
}
