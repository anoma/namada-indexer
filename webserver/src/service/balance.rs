use shared::balance::{Amount, Balance, DenominatedAmount};

use crate::appstate::AppState;
use crate::error::balance::BalanceError;
use crate::repository::balance::{BalanceRepo, BalanceRepoTrait};

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
    ) -> Result<Vec<Balance>, BalanceError> {
        let balances_db = self
            .balance_repo
            .get_address_balances(address)
            .await
            .map_err(BalanceError::Database)?;

        let balances: Vec<Balance> =
            balances_db.iter().cloned().map(Balance::from).collect();

        Ok(balances)
    }
}
