use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::tx_crawler_state;

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
