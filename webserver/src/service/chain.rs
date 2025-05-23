use bigdecimal::BigDecimal;
use shared::id::Id;
use shared::token::{IbcToken, Token};

use crate::appstate::AppState;
use crate::error::chain::ChainError;
use crate::repository::balance::{BalanceRepo, BalanceRepoTrait};
use crate::repository::chain::{ChainRepository, ChainRepositoryTrait};
use crate::response::chain::{CirculatingSupply, Parameters, TokenSupply};

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

    pub async fn find_latest_parameters(
        &self,
    ) -> Result<Parameters, ChainError> {
        let parameters = self
            .chain_repo
            .find_chain_parameters()
            .await
            .map(Parameters::from)
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

    pub async fn find_tokens(&self) -> Result<Vec<Token>, ChainError> {
        let tokens_db = self
            .chain_repo
            .find_tokens()
            .await
            .map_err(ChainError::Database)?;

        let tokens = tokens_db
            .into_iter()
            .map(|(token, ibc_token)| match ibc_token {
                Some(ibc_token) => Token::Ibc(IbcToken {
                    address: Id::Account(ibc_token.address),
                    trace: Id::IbcTrace(ibc_token.ibc_trace),
                }),
                None => Token::Native(Id::Account(token.address)),
            })
            .collect::<Vec<Token>>();

        Ok(tokens)
    }

    pub async fn get_token_supply(
        &self,
        address: String,
        epoch: Option<i32>,
    ) -> Result<Option<TokenSupply>, ChainError> {
        let maybe_token_supply_db = self
            .chain_repo
            .get_token_supply(address, epoch)
            .await
            .map_err(ChainError::Database)?;

        Ok(maybe_token_supply_db.map(|supply| TokenSupply {
            address: supply.address,
            total_supply: supply.total.to_string(),
            effective_supply: supply.effective.map(|s| s.to_string()),
        }))
    }

    pub async fn get_circulating_supply(
        &self,
        epoch: Option<i32>,
    ) -> Result<CirculatingSupply, ChainError> {
        // Native token address and address to exclude from circulating supply
        let native_token_address =
            "tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7";
        let excluded_address = "tnam1pgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkhgajr";

        // Get total supply of native token
        let total_supply_result = self
            .chain_repo
            .get_token_supply(native_token_address.to_string(), epoch)
            .await
            .map_err(ChainError::Database)?;

        let total_supply = total_supply_result.ok_or_else(|| {
            ChainError::Unknown("Native token supply not found".to_string())
        })?;

        // Get balance of excluded address
        let balance_repo = BalanceRepo::new(self.chain_repo.app_state.clone());
        let balances = balance_repo
            .get_address_balances(excluded_address.to_string())
            .await
            .map_err(ChainError::Database)?;

        // Find the balance of the native token for the excluded address
        let locked_amount = balances
            .iter()
            .find(|balance| balance.token == native_token_address)
            .map(|balance| balance.raw_amount.clone())
            .unwrap_or_else(|| BigDecimal::from(0));

        // Calculate circulating supply = total supply - locked amount
        let total_supply_amount = total_supply.total.clone();
        let circulating_amount = &total_supply_amount - &locked_amount;

        Ok(CirculatingSupply {
            total_supply: total_supply_amount.to_string(),
            locked_supply: locked_amount.to_string(),
            circulating_supply: circulating_amount.to_string(),
        })
    }
}
