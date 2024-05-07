use crate::block::BlockHeight;

use super::id::Id;

#[derive(Debug, Clone, Default)]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub proposer_address: String,
    pub timestamp: String,
    pub app_hash: Id,
}
