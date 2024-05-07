use axum::async_trait;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use orm::{schema::validators, validators::ValidatorDb};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct PosRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait PosRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn find_all_validators(&self) -> Result<Vec<ValidatorDb>, String>;
}

#[async_trait]
impl PosRepositoryTrait for PosRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_all_validators(&self) -> Result<Vec<ValidatorDb>, String> {
        let conn = self.app_state.get_db_connection().await;
        conn.interact(move |conn| {
            validators::table
                .select(ValidatorDb::as_select())
                .get_results(conn)
                .unwrap_or_default()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
