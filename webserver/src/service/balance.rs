use crate::appstate::AppState;
use crate::error::balance::BalanceError;
use crate::repository::balance::{BalanceRepo, BalanceRepoTrait};
use crate::response::balance::AddressBalance;

#[derive(Clone)]
pub struct BalanceService {
    pub balance_repo: BalanceRepo,
}

impl BalanceService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            balance_repo: BalanceRepo::new(app_state),
        }
    }

    pub async fn get_address_balances(
        &self,
        address: String,
    ) -> Result<Vec<AddressBalance>, BalanceError> {
        let balances = self
            .balance_repo
            .get_address_balances(address)
            .await
            .map_err(BalanceError::Database)?;

        Ok(balances.into_iter().map(AddressBalance::from).collect())
    }
}