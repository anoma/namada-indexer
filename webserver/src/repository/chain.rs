use axum::async_trait;
use diesel::dsl::max;
use diesel::{
    ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::crawler_state::{ChainCrawlerStateDb, CrawlerNameDb};
use orm::parameters::ParametersDb;
use orm::schema::{chain_parameters, crawler_state, ibc_token, token};
use orm::token::{IbcTokenDb, TokenDb};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct ChainRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait ChainRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_latest_height(&self) -> Result<i32, String>;

    async fn find_latest_epoch(&self) -> Result<i32, String>;

    async fn find_chain_parameters(&self) -> Result<ParametersDb, String>;

    async fn get_state(&self) -> Result<ChainCrawlerStateDb, String>;

    async fn find_tokens(
        &self,
    ) -> Result<Vec<(TokenDb, Option<IbcTokenDb>)>, String>;

    async fn get_native_token_address(&self) -> Result<String, String>;
}

#[async_trait]
impl ChainRepositoryTrait for ChainRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_latest_height(&self) -> Result<i32, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            crawler_state::dsl::crawler_state
                .filter(crawler_state::dsl::name.eq(CrawlerNameDb::Chain))
                .select(max(crawler_state::dsl::last_processed_block))
                .first::<Option<i32>>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
        .and_then(|x| x.ok_or("No block processed".to_string()))
    }

    async fn find_latest_epoch(&self) -> Result<i32, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            crawler_state::dsl::crawler_state
                .filter(crawler_state::dsl::name.eq(CrawlerNameDb::Chain))
                .select(max(crawler_state::dsl::last_processed_epoch))
                .first::<Option<i32>>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
        .and_then(|x| x.ok_or("No epoch processed".to_string()))
    }

    async fn get_state(&self) -> Result<ChainCrawlerStateDb, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            crawler_state::table
                .filter(crawler_state::dsl::name.eq(CrawlerNameDb::Chain))
                .select((
                    crawler_state::dsl::last_processed_block,
                    crawler_state::dsl::last_processed_epoch,
                    crawler_state::dsl::first_block_in_epoch,
                    crawler_state::dsl::timestamp,
                ))
                .first(conn)
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

    async fn get_native_token_address(&self) -> Result<String, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            chain_parameters::table
                .select(chain_parameters::columns::native_token_address)
                .first(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_tokens(
        &self,
    ) -> Result<Vec<(TokenDb, Option<IbcTokenDb>)>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            token::table
                .left_join(
                    ibc_token::table.on(token::address.eq(ibc_token::address)),
                )
                .select((
                    TokenDb::as_select(),
                    Option::<IbcTokenDb>::as_select(),
                ))
                .load::<(TokenDb, Option<IbcTokenDb>)>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
