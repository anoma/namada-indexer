use axum::async_trait;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::masp::MaspPoolDb;
use orm::schema::masp_pool_aggregate;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct MaspRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait MaspRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_all_aggregates(&self) -> Result<Vec<MaspPoolDb>, String>;

    async fn find_all_aggregates_by_token(
        &self,
        token: String,
    ) -> Result<Vec<MaspPoolDb>, String>;
}

#[async_trait]
impl MaspRepositoryTrait for MaspRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_all_aggregates(&self) -> Result<Vec<MaspPoolDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            masp_pool_aggregate::table
                .select(MaspPoolDb::as_select())
                .load(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_all_aggregates_by_token(
        &self,
        token: String,
    ) -> Result<Vec<MaspPoolDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            masp_pool_aggregate::table
                .filter(masp_pool_aggregate::dsl::token_address.eq(token))
                .select(MaspPoolDb::as_select())
                .load(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
