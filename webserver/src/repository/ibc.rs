use axum::async_trait;
use bigdecimal::BigDecimal;
use diesel::{
    ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::ibc::IbcAckDb;
use orm::schema::{ibc_ack, ibc_rate_limits, ibc_token_flows};

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
    ) -> Result<Vec<(String, String)>, String>;

    async fn get_token_flows(
        &self,
        token_address: Option<String>,
    ) -> Result<Vec<(String, String, String)>, String>;
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
        matching_token_address: Option<String>,
        matching_rate_limit: Option<BigDecimal>,
    ) -> Result<Vec<(String, String)>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            use diesel::Column;

            diesel::alias!(ibc_rate_limits as ibc_rate_limits_alias: IbcRateLimitsAlias);

            // NB: We're using a raw select because `CAST` is not available in the diesel dsl. :(
            let select_statement = diesel::dsl::sql::<(diesel::sql_types::Text, diesel::sql_types::Text)>(
                &format!(
                    "{}, CAST({} AS TEXT)",
                    ibc_rate_limits::dsl::address::NAME,
                    ibc_rate_limits::dsl::throughput_limit::NAME,
                ),
            );

            let max_epoch_where_clause =
                ibc_rate_limits::dsl::epoch.nullable().eq(ibc_rate_limits_alias
                    .select(diesel::dsl::max(ibc_rate_limits_alias.field(ibc_rate_limits::dsl::epoch)))
                    .single_value());

            match (matching_token_address, matching_rate_limit) {
                (None, None) => ibc_rate_limits::table
                    .filter(max_epoch_where_clause)
                    .select(select_statement)
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

    async fn get_token_flows(
        &self,
        matching_token_address: Option<String>,
    ) -> Result<Vec<(String, String, String)>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            use diesel::Column;

            diesel::alias!(ibc_token_flows as ibc_token_flows_alias: IbcTokenFlowsAlias);

            // NB: We're using a raw select because `CAST` is not available in the diesel dsl. :(
            let select_statement =
                diesel::dsl::sql::<(
                    diesel::sql_types::Text,
                    diesel::sql_types::Text,
                    diesel::sql_types::Text
                )>(
                    &format!(
                        "{}, CAST({} AS TEXT), CAST({} AS TEXT)",
                        ibc_token_flows::dsl::address::NAME,
                        ibc_token_flows::dsl::withdraw::NAME,
                        ibc_token_flows::dsl::deposit::NAME,
                    ),
                );

            let max_epoch_where_clause =
                ibc_token_flows::dsl::epoch.nullable().eq(ibc_token_flows_alias
                    .select(diesel::dsl::max(ibc_token_flows_alias.field(ibc_token_flows::dsl::epoch)))
                    .single_value());

            match matching_token_address {
                None => ibc_token_flows::table
                    .filter(max_epoch_where_clause)
                    .select(select_statement)
                    .load(conn),
                Some(token) => ibc_token_flows::table
                    .filter(ibc_token_flows::dsl::address.eq(&token))
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
