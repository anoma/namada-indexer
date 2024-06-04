use crate::appstate::AppState;
use crate::error::balance::BalanceError;
use crate::repository::balance::{BalanceRepo, BalanceRepoTrait};
use crate::response::balance::AddressBalance;

use super::utils::raw_amount_to_nam;

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

        // TODO: temporary solution as we only store NAM balances
        let denominated_balances: Vec<AddressBalance> = balances
            .iter()
            .cloned()
            .map(|balance| AddressBalance {
                token_address: balance.token,
                balance: raw_amount_to_nam(balance.raw_amount),
            })
            .collect();

        Ok(denominated_balances)
    }
}
