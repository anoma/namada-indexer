use crate::appstate::AppState;
use crate::error::gas::GasError;
use crate::repository::gas::{GasRepository, GasRepositoryTrait};
use crate::response::gas::{Gas, GasPrice};

#[derive(Clone)]
pub struct GasService {
    gas_repo: GasRepository,
}

impl GasService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            gas_repo: GasRepository::new(app_state),
        }
    }

    pub async fn get_gas(&self) -> Vec<Gas> {
        self.gas_repo
            .get_gas()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Gas::from)
            .collect()
    }

    pub async fn get_gas_price_by_token(
        &self,
        token: String,
    ) -> Result<Vec<GasPrice>, GasError> {
        self.gas_repo
            .find_gas_price_by_token(token)
            .await
            .map_err(GasError::Database)
            .map(|r| r.iter().cloned().map(GasPrice::from).collect())
    }

    pub async fn get_all_gas_prices(&self) -> Result<Vec<GasPrice>, GasError> {
        self.gas_repo
            .find_all_gas_prices()
            .await
            .map_err(GasError::Database)
            .map(|r| r.iter().cloned().map(GasPrice::from).collect())
    }
}
