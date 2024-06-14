use axum::async_trait;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::gas::{GasDb, GasPriceDb};
use orm::schema::{gas, gas_price};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct GasRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait GasRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_gas_by_token(
        &self,
        token: String,
    ) -> Result<Vec<GasDb>, String>;

    async fn find_gas_price_by_token(
        &self,
        token: String,
    ) -> Result<GasPriceDb, String>;
}

#[async_trait]
impl GasRepositoryTrait for GasRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_gas_by_token(
        &self,
        token: String,
    ) -> Result<Vec<GasDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            gas::table
                .filter(gas::dsl::token.eq(token))
                .select(GasDb::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_gas_price_by_token(
        &self,
        token: String,
    ) -> Result<GasPriceDb, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            gas_price::table
                .filter(gas_price::token.eq(token))
                .select(GasPriceDb::as_select())
                .first(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
