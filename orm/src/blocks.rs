use diesel::{Insertable, Queryable, Selectable};
use shared::block::Block;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;

use crate::schema::blocks;

#[derive(Insertable, Clone, Queryable, Selectable, Debug)]
#[diesel(table_name = blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockInsertDb {
    pub height: i32,
    pub hash: String,
    pub app_hash: String,
    pub timestamp: chrono::NaiveDateTime,
    pub proposer: String,
    pub epoch: i32,
}

pub type BlockDb = BlockInsertDb;

impl From<(Block, TendermintBlockResponse)> for BlockInsertDb {
    fn from(
        (block, tm_block_response): (Block, TendermintBlockResponse),
    ) -> Self {
        let timestamp = chrono::DateTime::from_timestamp(
            tm_block_response.block.header.time.unix_timestamp(),
            0,
        )
        .expect("Invalid timestamp")
        .naive_utc();

        Self {
            height: block.header.height as i32,
            hash: block.hash.to_string(),
            app_hash: block.header.app_hash.to_string(),
            timestamp,
            proposer: block.header.proposer_address,
            epoch: block.epoch as i32,
        }
    }
}

impl BlockInsertDb {
    pub fn fake(height: i32) -> Self {
        Self {
            height,
            hash: height.to_string(), // fake hash but ensures uniqueness with height
            app_hash: "fake_app_hash".to_string(), // doesn't require uniqueness
            timestamp: chrono::DateTime::from_timestamp(0, 0)
                .unwrap()
                .naive_utc(),
            proposer: "fake_proposer".to_string(),
            epoch: 0,
        }
    }
}
