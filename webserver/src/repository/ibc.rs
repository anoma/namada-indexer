use axum::async_trait;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::ibc::IbcAckDb;
use orm::schema::ibc_ack;

use crate::appstate::AppState;

#[derive(Clone)]
pub struct IbcRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait IbcRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_ibc_ack(
        &self,
        id: String,
    ) -> Result<Option<IbcAckDb>, String>;
}

#[async_trait]
impl IbcRepositoryTrait for IbcRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_ibc_ack(
        &self,
        id: String,
    ) -> Result<Option<IbcAckDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            ibc_ack::table
                .filter(ibc_ack::dsl::tx_hash.eq(id))
                .select(IbcAckDb::as_select())
                .first(conn)
                .ok()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
