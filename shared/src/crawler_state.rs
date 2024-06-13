use crate::block::{BlockHeight, Epoch};

#[derive(Debug, Clone, Default)]
pub struct CrawlerState {
    pub height: BlockHeight,
    pub epoch: Epoch,
    pub timestamp: i64,
}

impl CrawlerState {
    pub fn new(height: BlockHeight, epoch: Epoch, timestamp: i64) -> Self {
        Self {
            height,
            epoch,
            timestamp,
        }
    }
}
