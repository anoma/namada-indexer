use axum::async_trait;
use diesel::{dsl::max, QueryDsl, RunQueryDsl};
use orm::schema::block_crawler_state;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct ChainRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait ChainRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_latest_height(
        &self,
    ) -> Result<Option<i32>, String>;
}

#[async_trait]
impl ChainRepositoryTrait for ChainRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_latest_height(
        &self,
    ) -> Result<Option<i32>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            block_crawler_state::dsl::block_crawler_state
                .select(max(block_crawler_state::dsl::height))
                .first::<Option<i32>>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
