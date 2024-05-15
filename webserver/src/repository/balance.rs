use axum::async_trait;
use orm::balances::BalanceDb;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct BalanceRepo {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait BalanceRepoTrait {
    fn new(app_state: AppState) -> Self;

    async fn get_address_balances(
        &self,
        address: String,
    ) -> Result<Vec<BalanceDb>, String>;
}

#[async_trait]
impl BalanceRepoTrait for BalanceRepo {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_address_balances(
        &self,
        address: String,
    ) -> Result<Vec<BalanceDb>, String> {
        todo!()
    }
}
