use crate::appstate::AppState;
use crate::error::chain::ChainError;
use crate::repository::chain::{ChainRepository, ChainRepositoryTrait};
use crate::response::parameters::Parameters;

#[derive(Clone)]
pub struct ChainService {
    chain_repo: ChainRepository,
}

impl ChainService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            chain_repo: ChainRepository::new(app_state),
        }
    }

    pub async fn find_latest_height(&self) -> u64 {
        self.chain_repo
            .find_latest_height()
            .await
            .unwrap()
            .unwrap_or_default() as u64
    }

    pub async fn find_latest_parameters(
        &self,
    ) -> Result<Parameters, ChainError> {
        let epoch = self
            .chain_repo
            .find_latest_epoch()
            .await
            .map_err(ChainError::Database)?
            .expect("latest epoch not found");

        let parameters = self
            .chain_repo
            .find_chain_parameters(epoch)
            .await
            .map(|parameters| Parameters {
                unbonding_length: parameters.unbonding_length as u64,
                pipeline_length: parameters.pipeline_length as u64,
                epochs_per_year: parameters.epochs_per_year as u64,
            })
            .map_err(ChainError::Database)?;

        Ok(parameters)
    }
}