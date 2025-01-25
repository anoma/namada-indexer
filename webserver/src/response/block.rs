use orm::blocks::BlockDb;
use orm::transactions::WrapperTransactionDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub height: i32,
    pub hash: Option<String>,
    pub app_hash: Option<String>,
    pub timestamp: Option<String>,
    pub proposer: Option<String>,
    pub transactions: Vec<String>,
    pub parent_hash: Option<String>,
    pub epoch: Option<String>,
}

impl Block {
    pub fn from(
        block_db: BlockDb,
        prev_block_db: Option<BlockDb>,
        transactions: Vec<WrapperTransactionDb>,
    ) -> Self {
        Self {
            height: block_db.height,
            hash: block_db.hash,
            app_hash: block_db.app_hash,
            timestamp: block_db
                .timestamp
                .map(|t| t.and_utc().timestamp().to_string()),
            proposer: block_db.proposer,
            transactions: transactions
                .into_iter()
                .map(|wrapper| wrapper.id.to_lowercase())
                .collect(),
            parent_hash: prev_block_db
                .map(|block| block.app_hash)
                .unwrap_or(None),
            epoch: block_db.epoch.map(|e| e.to_string()),
        }
    }
}
