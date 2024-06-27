use shared::balance::{Amount, DenominatedAmount};

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

        // TODO: temporary solution as we only store NAM balances
        let denominated_balances: Vec<AddressBalance> = balances
            .iter()
            .cloned()
            .map(|balance| AddressBalance {
                token_address: balance.token,
                // TODO: change native to new once we support multiple tokens
                balance: DenominatedAmount::native(Amount::from(
                    balance.raw_amount,
                ))
                .to_string_precise(),
            })
            .collect();

        Ok(denominated_balances)
    }
}
