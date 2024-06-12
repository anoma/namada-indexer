use crate::appstate::AppState;
use crate::repository::gas::{GasRepository, GasRepositoryTrait};
use crate::response::gas::Gas;

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

    pub async fn get_gas_by_token(&self, token: String) -> Vec<Gas> {
        self.gas_repo
            .find_gas_by_token(token)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Gas::from)
            .collect()
    }
}
