use super::id::Id;
use crate::block::BlockHeight;

#[derive(Debug, Clone, Default)]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub proposer_address_tm: String,
    pub proposer_address_namada: Option<String>,
    pub timestamp: i64,
    pub app_hash: Id,
}
