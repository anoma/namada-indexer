use shared::checksums::Checksums;
use shared::genesis::Genesis;
use shared::parameters::{EpochSwitchBlocksDelay, Parameters};

use crate::appstate::AppState;
use crate::error::chain::ChainError;
use crate::repository::chain::{ChainRepository, ChainRepositoryTrait};

#[derive(Clone)]
pub struct ChainService {
    chain_repo: ChainRepository,
}

pub type LatestParameters =
    (Parameters, Genesis, Checksums, EpochSwitchBlocksDelay);

impl ChainService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            chain_repo: ChainRepository::new(app_state),
        }
    }

    pub async fn find_latest_parameters(
        &self,
    ) -> Result<LatestParameters, ChainError> {
        let parameters = self
            .chain_repo
            .find_chain_parameters()
            .await
            .map(LatestParameters::from)
            .map_err(ChainError::Database)?;

        Ok(parameters)
    }

    pub async fn find_last_processed_block(&self) -> Result<i32, ChainError> {
        self.chain_repo
            .find_latest_height()
            .await
            .map_err(ChainError::Database)
    }

    pub async fn find_last_processed_epoch(&self) -> Result<i32, ChainError> {
        self.chain_repo
            .find_latest_epoch()
            .await
            .map_err(ChainError::Database)
    }
}
