use shared::balance::{Amount, DenominatedAmount};

use crate::appstate::AppState;
use crate::error::balance::BalanceError;
use crate::repository::balance::{BalanceRepo, BalanceRepoTrait};
use crate::repository::chain::{ChainRepository, ChainRepositoryTrait};
use crate::response::balance::AddressBalance;

#[derive(Clone)]
pub struct BalanceService {
    pub balance_repo: BalanceRepo,
    pub chain_repo: ChainRepository,
}

impl BalanceService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            balance_repo: BalanceRepo::new(app_state.clone()),
            chain_repo: ChainRepository::new(app_state),
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
        let native_token = self
            .chain_repo
            .get_native_token_address()
            .await
            .map_err(BalanceError::Database)?;

        let denominated_balances: Vec<AddressBalance> = balances
            .iter()
            .cloned()
            .map(|balance| {
                let token_address = balance.token;
                let amount = if token_address == native_token {
                    DenominatedAmount::native(Amount::from(balance.raw_amount))
                        .to_string_precise()
                } else {
                    Amount::from(balance.raw_amount).to_string()
                };

                AddressBalance {
                    token_address,
                    balance: amount,
                }
            })
            .collect();

        Ok(denominated_balances)
    }
}
