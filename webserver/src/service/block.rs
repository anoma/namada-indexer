use crate::appstate::AppState;
use crate::error::block::BlockError;
use crate::repository::block::{BlockRepository, BlockRepositoryTrait};
use crate::repository::tranasaction::{
    TransactionRepository, TransactionRepositoryTrait,
};
use crate::response::block::Block;

#[derive(Clone)]
pub struct BlockService {
    block_repo: BlockRepository,
    transaction_repo: TransactionRepository,
}

impl BlockService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            block_repo: BlockRepository::new(app_state.clone()),
            transaction_repo: TransactionRepository::new(app_state),
        }
    }

    pub async fn get_block_by_height(
        &self,
        height: i32,
    ) -> Result<Block, BlockError> {
        let block = self
            .block_repo
            .find_block_by_height(height)
            .await
            .map_err(BlockError::Database)?;
        let block = block.ok_or(BlockError::NotFound(
            "height".to_string(),
            height.to_string(),
        ))?;
        let prev_block = if let Some(block_height) = block.height.checked_sub(1)
        {
            self.block_repo
                .find_block_by_height(block_height)
                .await
                .map_err(BlockError::Database)?
        } else {
            None
        };

        let transactions = self
            .transaction_repo
            .find_txs_by_block_height(block.height)
            .await
            .map_err(BlockError::Database)?;

        Ok(Block::from(block, prev_block, transactions))
    }

    pub async fn get_block_by_timestamp(
        &self,
        timestamp: i64,
    ) -> Result<Block, BlockError> {
        let block = self
            .block_repo
            .find_block_by_timestamp(timestamp)
            .await
            .map_err(BlockError::Database)?;

        let block = block.ok_or(BlockError::NotFound(
            "timestamp".to_string(),
            timestamp.to_string(),
        ))?;
        let prev_block = if let Some(block_height) = block.height.checked_sub(1)
        {
            self.block_repo
                .find_block_by_height(block_height)
                .await
                .map_err(BlockError::Database)?
        } else {
            None
        };

        let transactions = self
            .transaction_repo
            .find_txs_by_block_height(block.height)
            .await
            .map_err(BlockError::Database)?;

        Ok(Block::from(block, prev_block, transactions))
    }
}
