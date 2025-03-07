use crate::block::{BlockHeight, Epoch};

pub enum CrawlerName {
    Chain,
    Governance,
    Parameters,
    Pos,
    Rewards,
    Transactions,
    Cometbft,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChainCrawlerState {
    pub last_processed_block: BlockHeight,
    pub last_processed_epoch: Epoch,
    pub first_block_in_epoch: Epoch,
    pub timestamp: i64,
}

#[derive(Debug)]
pub struct BlockCrawlerState {
    pub last_processed_block: BlockHeight,
    pub timestamp: i64,
}

#[derive(Debug)]
pub struct EpochCrawlerState {
    pub last_processed_epoch: Epoch,
    pub timestamp: i64,
}

#[derive(Debug)]
pub struct IntervalCrawlerState {
    pub timestamp: i64,
}

#[derive(Debug)]
pub struct CometbftCrawlerState {
    pub last_processed_block: BlockHeight,
    pub timestamp: i64,
}
