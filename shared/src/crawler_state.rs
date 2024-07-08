use crate::block::{BlockHeight, Epoch};

pub enum CrawlerName {
    Chain,
    Governance,
    Parameters,
    Pos,
    Rewards,
    Transactions,
}

pub struct BlockCrawlerState {
    pub last_processed_block: BlockHeight,
    pub last_processed_epoch: Epoch,
    pub timestamp: i64,
}

pub struct EpochCrawlerState {
    pub last_processed_epoch: Epoch,
    pub timestamp: i64,
}

pub struct IntervalCrawlerState {
    pub timestamp: i64,
}
