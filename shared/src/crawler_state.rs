use crate::block::{BlockHeight, Epoch};

#[derive(Debug, Clone, Default)]
pub struct CrawlerState {
    pub height: BlockHeight,
    pub epoch: Epoch,
}

impl CrawlerState {
    pub fn new(height: BlockHeight, epoch: Epoch) -> Self {
        Self { height, epoch }
    }
}
