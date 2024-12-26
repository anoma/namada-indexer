use orm::blocks::BlockDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub height: i32,
    pub hash: Option<String>,
    pub app_hash: Option<String>,
    pub timestamp: Option<String>,
    pub proposer: Option<String>,
    pub epoch: Option<String>,
}

impl From<BlockDb> for Block {
    fn from(block_db: BlockDb) -> Self {
        Self {
            height: block_db.height,
            hash: block_db.hash,
            app_hash: block_db.app_hash,
            timestamp: block_db
                .timestamp
                .map(|t| t.and_utc().timestamp().to_string()),
            proposer: block_db.proposer,
            epoch: block_db.epoch.map(|e| e.to_string()),
        }
    }
}
