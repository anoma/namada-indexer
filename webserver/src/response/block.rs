use orm::block::BlockDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub height: i32,
    pub timestamp: String,
}

impl From<BlockDb> for Block {
    fn from(block_db: BlockDb) -> Self {
        Self {
            height: block_db.height,
            timestamp: block_db.time.and_utc().timestamp().to_string(),
        }
    }
}
