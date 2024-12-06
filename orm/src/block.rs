use std::str::FromStr;

use diesel::{Insertable, Queryable, Selectable};
use shared::block::Block;

use crate::schema::block;

#[derive(Clone, Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = block)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockDb {
    pub height: i32,
    pub time: chrono::NaiveDateTime,
}

impl From<Block> for BlockDb {
    fn from(block: Block) -> Self {
        let datetime =
            chrono::DateTime::<chrono::Utc>::from_str(&block.header.timestamp)
                .expect("Could not parse timestamp");

        let timestamp =
            chrono::DateTime::from_timestamp(datetime.timestamp(), 0)
                .expect("Invalid timestamp")
                .naive_utc();

        Self {
            height: block.header.height as i32,
            time: timestamp,
        }
    }
}
