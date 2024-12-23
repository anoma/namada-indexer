use axum::async_trait;
use bigdecimal::BigDecimal;
use diesel::dsl::{avg, count, max, min};
use diesel::sql_types::{BigInt, Integer, Nullable, Numeric};
use diesel::{
    ExpressionMethods, IntoSql, JoinOnDsl, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::gas::{GasDb, GasPriceDb};
use orm::schema::{gas, gas_estimations, gas_price, wrapper_transactions};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct GasRepository {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait GasRepositoryTrait {
    fn new(app_state: AppState) -> Self;

    async fn get_gas(&self) -> Result<Vec<GasDb>, String>;

    async fn find_gas_price_by_token(
        &self,
        token: String,
    ) -> Result<Vec<GasPriceDb>, String>;

    async fn find_all_gas_prices(&self) -> Result<Vec<GasPriceDb>, String>;

    #[allow(clippy::too_many_arguments)]
    async fn find_gas_estimates(
        &self,
        bond: u64,
        redelegate: u64,
        claim_rewards: u64,
        unbond: u64,
        transparent_transfer: u64,
        shielded_transfer: u64,
        shielding_transfer: u64,
        unshielding_transfer: u64,
        vote: u64,
        ibc: u64,
        withdraw: u64,
        reveal_pk: u64,
        signatures: u64,
        tx_size: u64,
    ) -> Result<(Option<i32>, Option<i32>, Option<BigDecimal>, i64), String>;
}

#[async_trait]
impl GasRepositoryTrait for GasRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn get_gas(&self) -> Result<Vec<GasDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            gas::table.select(GasDb::as_select()).get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_gas_price_by_token(
        &self,
        token: String,
    ) -> Result<Vec<GasPriceDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            gas_price::table
                .filter(gas_price::token.eq(token))
                .select(GasPriceDb::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_all_gas_prices(&self) -> Result<Vec<GasPriceDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            gas_price::table
                .select(GasPriceDb::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    #[allow(clippy::too_many_arguments)]
    async fn find_gas_estimates(
        &self,
        bond: u64,
        redelegate: u64,
        claim_rewards: u64,
        unbond: u64,
        transparent_transfer: u64,
        shielded_transfer: u64,
        shielding_transfer: u64,
        unshielding_transfer: u64,
        vote: u64,
        ibc: u64,
        withdraw: u64,
        reveal_pk: u64,
        signatures: u64,
        tx_size: u64,
    ) -> Result<(Option<i32>, Option<i32>, Option<BigDecimal>, i64), String>
    {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            gas_estimations::table
                .filter(gas_estimations::dsl::bond.eq(bond as i32))
                .filter(
                    gas_estimations::dsl::redelegation.eq(redelegate as i32),
                )
                .filter(
                    gas_estimations::dsl::claim_rewards
                        .eq(claim_rewards as i32),
                )
                .filter(gas_estimations::dsl::unbond.eq(unbond as i32))
                .filter(
                    gas_estimations::dsl::transparent_transfer
                        .eq(transparent_transfer as i32),
                )
                .filter(
                    gas_estimations::dsl::shielded_transfer
                        .eq(shielded_transfer as i32),
                )
                .filter(
                    gas_estimations::dsl::shielding_transfer
                        .eq(shielding_transfer as i32),
                )
                .filter(
                    gas_estimations::dsl::unshielding_transfer
                        .eq(unshielding_transfer as i32),
                )
                .filter(gas_estimations::dsl::vote_proposal.eq(vote as i32))
                .filter(gas_estimations::dsl::ibc_msg_transfer.eq(ibc as i32))
                .filter(gas_estimations::dsl::withdraw.eq(withdraw as i32))
                .filter(gas_estimations::dsl::reveal_pk.eq(reveal_pk as i32))
                .filter(gas_estimations::dsl::signatures.ge(signatures as i32))
                .filter(gas_estimations::dsl::tx_size.ge(tx_size as i32))
                .inner_join(
                    wrapper_transactions::table
                        .on(gas_estimations::dsl::wrapper_id
                            .eq(wrapper_transactions::dsl::id)),
                )
                .limit(100)
                .select((
                    min(wrapper_transactions::dsl::gas_used)
                        .into_sql::<Nullable<Integer>>(),
                    max(wrapper_transactions::dsl::gas_used)
                        .into_sql::<Nullable<Integer>>(),
                    avg(wrapper_transactions::dsl::gas_used)
                        .into_sql::<Nullable<Numeric>>(),
                    count(wrapper_transactions::dsl::gas_used)
                        .into_sql::<BigInt>(),
                ))
                .get_result::<(Option<i32>, Option<i32>, Option<BigDecimal>, i64)>(
                    conn,
                )
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
