use crate::appstate::AppState;
use crate::error::block::BlockError;
use crate::repository::block::{BlockRepository, BlockRepositoryTrait};
use crate::response::block::Block;

#[derive(Clone)]
pub struct BlockService {
    block_repo: BlockRepository,
}

impl BlockService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            block_repo: BlockRepository::new(app_state),
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

        Ok(Block::from(block))
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

        Ok(Block::from(block))
    }
}
