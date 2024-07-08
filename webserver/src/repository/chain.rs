use axum::async_trait;
use diesel::dsl::max;
use diesel::{
    ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::crawler_state::BlockCrawlerStateDb;
use orm::parameters::ParametersDb;
use orm::schema::{chain_parameters, crawler_state};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct ChainRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait ChainRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_latest_height(&self) -> Result<Option<i32>, String>;

    async fn find_latest_epoch(&self) -> Result<Option<i32>, String>;

    async fn find_chain_parameters(&self) -> Result<ParametersDb, String>;

    async fn get_state(&self) -> Result<BlockCrawlerStateDb, String>;
}

#[async_trait]
impl ChainRepositoryTrait for ChainRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_latest_height(&self) -> Result<Option<i32>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            crawler_state::dsl::crawler_state
                .select(max(crawler_state::dsl::last_processed_block))
                .first::<Option<i32>>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_latest_epoch(&self) -> Result<Option<i32>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            crawler_state::dsl::crawler_state
                .select(max(crawler_state::dsl::last_processed_epoch))
                .first::<Option<i32>>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn get_state(&self) -> Result<BlockCrawlerStateDb, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            let (state1, state2) = diesel::alias!(
                crawler_state as state1,
                crawler_state as state2
            );
            let subquery = state1
                .select(max(state1.field(crawler_state::last_processed_block)))
                .single_value();

            state2
                .filter(
                    state2
                        .field(crawler_state::last_processed_block)
                        .nullable()
                        .eq(subquery),
                )
                .select(state2.fields((
                    crawler_state::last_processed_block,
                    crawler_state::last_processed_epoch,
                    crawler_state::timestamp,
                )))
                .first::<BlockCrawlerStateDb>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_chain_parameters(&self) -> Result<ParametersDb, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            chain_parameters::table
                .select(ParametersDb::as_select())
                .first(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
