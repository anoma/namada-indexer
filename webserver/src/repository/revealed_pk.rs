use axum::async_trait;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::revealed_pk::{RevealedPkDb, RevealedPkInsertDb};
use orm::schema::revealed_pk;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct RevealedPkRepo {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait PkRepoTrait {
    fn new(app_state: AppState) -> Self;

    async fn get_revealed_pk_by_address(
        &self,
        address: String,
    ) -> Result<Option<RevealedPkDb>, String>;

    async fn insert_revealed_pk(
        &self,
        address: RevealedPkInsertDb,
    ) -> Result<(), String>;
}

#[async_trait]
impl PkRepoTrait for RevealedPkRepo {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_revealed_pk_by_address(
        &self,
        address: String,
    ) -> Result<Option<RevealedPkDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            revealed_pk::table
                .filter(revealed_pk::dsl::address.eq(address))
                .select(RevealedPkDb::as_select())
                .first(conn)
                .ok()
        })
        .await
        .map_err(|e| e.to_string())
    }

    async fn insert_revealed_pk(
        &self,
        revealed_pk: RevealedPkInsertDb,
    ) -> Result<(), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| -> Result<(), String> {
            diesel::insert_into(revealed_pk::table)
                .values::<Vec<RevealedPkInsertDb>>(vec![revealed_pk])
                .on_conflict_do_nothing()
                .execute(conn)
                .map_err(|e| e.to_string())?;

            Ok(())
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
