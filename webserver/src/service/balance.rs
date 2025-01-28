use shared::balance::Amount;

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

        let tokens = self
            .balance_repo
            .get_all_token()
            .await
            .map_err(BalanceError::Database)?;

        // TODO: temporary solution as we only store NAM balances
        let denominated_balances = tokens
            .into_iter()
            .map(|token| AddressBalance {
                token_address: token.address.clone(),
                min_denom_amount: balances
                    .iter()
                    .find(|&balance| balance.token.eq(&token.address))
                    .cloned()
                    .map(|balance| Amount::from(balance.raw_amount).to_string())
                    .unwrap_or_else(|| Amount::zero().to_string()),
            })
            .collect();

        Ok(denominated_balances)
    }
}
