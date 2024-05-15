use axum::async_trait;
use diesel::{QueryDsl, SelectableHelper};
use orm::schema::validators;
use orm::validators::ValidatorDb;

use super::utils::Paginate;
use crate::appstate::AppState;

#[derive(Clone)]
pub struct PosRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait PosRepositoryTrait {
    fn new(app_state: AppState) -> Self;
    async fn find_all_validators(
        &self,
        page: i64,
    ) -> Result<(Vec<ValidatorDb>, i64), String>;
}

#[async_trait]
impl PosRepositoryTrait for PosRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_all_validators(
        &self,
        page: i64,
    ) -> Result<(Vec<ValidatorDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .select(ValidatorDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn)
                .unwrap()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
