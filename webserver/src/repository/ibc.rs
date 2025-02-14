use axum::async_trait;
use bigdecimal::BigDecimal;
use diesel::{
    ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::ibc::IbcAckDb;
use orm::schema::{ibc_ack, ibc_rate_limits};

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

    async fn get_throughput_limits(
        &self,
        token_address: Option<String>,
        matching_rate_limit: Option<BigDecimal>,
    ) -> Result<Vec<(String, BigDecimal)>, String>;
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

    async fn get_throughput_limits(
        &self,
        token_address: Option<String>,
        matching_rate_limit: Option<BigDecimal>,
    ) -> Result<Vec<(String, BigDecimal)>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            diesel::alias!(ibc_rate_limits as ibc_rate_limits_alias: IbcRateLimitsAlias);

            let select_statement = (
                ibc_rate_limits::dsl::address,
                ibc_rate_limits::dsl::throughput_limit,
            );

            let max_epoch_where_clause =
                ibc_rate_limits::dsl::epoch.nullable().eq(ibc_rate_limits_alias
                    .select(diesel::dsl::max(ibc_rate_limits_alias.field(ibc_rate_limits::dsl::epoch)))
                    .single_value());

            match (token_address, matching_rate_limit) {
                (None, None) => ibc_rate_limits::table
                    .select(select_statement)
                    .filter(max_epoch_where_clause)
                    .load(conn),
                (Some(token), None) => ibc_rate_limits::table
                    .filter(ibc_rate_limits::dsl::address.eq(&token))
                    .filter(max_epoch_where_clause)
                    .select(select_statement)
                    .load(conn),
                (None, Some(limit)) => ibc_rate_limits::table
                    .filter(ibc_rate_limits::dsl::throughput_limit.eq(&limit))
                    .filter(max_epoch_where_clause)
                    .select(select_statement)
                    .load(conn),
                (Some(token), Some(limit)) => ibc_rate_limits::table
                    .filter(
                        ibc_rate_limits::dsl::throughput_limit
                            .eq(&limit)
                    )
                    .filter(ibc_rate_limits::dsl::address.eq(&token))
                    .filter(max_epoch_where_clause)
                    .select(select_statement)
                    .load(conn),
            }
            .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
