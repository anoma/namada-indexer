use orm::crawler_state::CrawlerStateInsertDb;

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

    pub fn to_crawler_state_db(&self) -> CrawlerStateInsertDb {
        CrawlerStateInsertDb {
            height: self.height as i32,
            epoch: self.epoch as i32,
        }
    }
}
