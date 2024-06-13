use axum::async_trait;
use diesel::dsl::max;
use diesel::{
    ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::block_crawler_state::BlockCrawlerStateDb;
use orm::parameters::ParametersDb;
use orm::schema::{block_crawler_state, chain_parameters};

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

    async fn get_chain_state(&self) -> Result<BlockCrawlerStateDb, String>;
}

#[async_trait]
impl ChainRepositoryTrait for ChainRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_latest_height(&self) -> Result<Option<i32>, String> {
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

    async fn find_latest_epoch(&self) -> Result<Option<i32>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            block_crawler_state::dsl::block_crawler_state
                .select(max(block_crawler_state::dsl::epoch))
                .first::<Option<i32>>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn get_chain_state(&self) -> Result<BlockCrawlerStateDb, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            let (state1, state2) = diesel::alias!(
                block_crawler_state as state1,
                block_crawler_state as state2
            );
            let subquery = state1
                .select(max(state1.field(block_crawler_state::height)))
                .single_value();

            state2
                .filter(
                    state2
                        .field(block_crawler_state::height)
                        .nullable()
                        .eq(subquery),
                )
                .select(state2.fields(block_crawler_state::all_columns))
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
