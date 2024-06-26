use axum::async_trait;
use bigdecimal::BigDecimal;
use diesel::dsl::sum;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::bond::BondDb;
use orm::pos_rewards::PoSRewardDb;
use orm::schema::{bonds, pos_rewards, unbonds, validators};
use orm::unbond::UnbondDb;
use orm::validators::{ValidatorDb, ValidatorStateDb};

use super::utils::Paginate;
use crate::appstate::AppState;
use crate::dto::pos::MyValidatorKindDto;

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
        states: Vec<ValidatorStateDb>,
    ) -> Result<(Vec<ValidatorDb>, i64), String>;

    async fn find_validators_by_addresses(
        &self,
        page: i64,
        addreses: Vec<String>,
        kind: MyValidatorKindDto,
    ) -> Result<(Vec<ValidatorDb>, i64), String>;

    async fn find_validator_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ValidatorDb>, String>;

    async fn find_merged_bonds_by_address(
        &self,
        address: String,
        page: i64,
    ) -> Result<(Vec<(String, ValidatorDb, Option<BigDecimal>)>, i64), String>;

    async fn find_bonds_by_address(
        &self,
        address: String,
        page: i64,
    ) -> Result<(Vec<(ValidatorDb, BondDb)>, i64), String>;

    async fn find_unbonds_by_address(
        &self,
        address: String,
        page: i64,
    ) -> Result<(Vec<(ValidatorDb, UnbondDb)>, i64), String>;

    async fn find_withdraws_by_address(
        &self,
        address: String,
        current_epoch: i32,
        page: i64,
    ) -> Result<(Vec<(ValidatorDb, UnbondDb)>, i64), String>;

    async fn find_rewards_by_address(
        &self,
        address: String,
    ) -> Result<Vec<PoSRewardDb>, String>;

    async fn get_total_voting_power(&self) -> Result<Option<i64>, String>;
}

#[async_trait]
impl PosRepositoryTrait for PosRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_all_validators(
        &self,
        page: i64,
        states: Vec<ValidatorStateDb>,
    ) -> Result<(Vec<ValidatorDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .filter(validators::dsl::state.eq_any(states))
                .select(ValidatorDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_validators_by_addresses(
        &self,
        page: i64,
        addreses: Vec<String>,
        kind: MyValidatorKindDto,
    ) -> Result<(Vec<ValidatorDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| match kind {
            MyValidatorKindDto::WithBonds => validators::table
                .inner_join(bonds::table)
                .filter(bonds::dsl::address.eq_any(addreses.clone()))
                .select(ValidatorDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn),

            MyValidatorKindDto::WithUnbonds => validators::table
                .inner_join(unbonds::table)
                .filter(unbonds::dsl::address.eq_any(addreses.clone()))
                .select(ValidatorDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn),
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_validator_by_id(
        &self,
        validator_id: i32,
    ) -> Result<Option<ValidatorDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .filter(validators::dsl::id.eq(validator_id))
                .select(ValidatorDb::as_select())
                .first(conn)
                .ok()
        })
        .await
        .map_err(|e| e.to_string())
    }

    async fn find_bonds_by_address(
        &self,
        address: String,
        page: i64,
    ) -> Result<(Vec<(ValidatorDb, BondDb)>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .inner_join(bonds::table)
                .filter(bonds::dsl::address.eq(address))
                .select((validators::all_columns, bonds::all_columns))
                .paginate(page)
                .load_and_count_pages::<(ValidatorDb, BondDb)>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_merged_bonds_by_address(
        &self,
        address: String,
        page: i64,
    ) -> Result<(Vec<(String, ValidatorDb, Option<BigDecimal>)>, i64), String>
    {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .inner_join(bonds::table)
                .filter(bonds::address.eq(address.clone()))
                .group_by((bonds::address, validators::id))
                .select((
                    bonds::address,
                    validators::all_columns,
                    sum(bonds::raw_amount),
                ))
                .paginate(page)
                // TODO: this is ok for now, create mixed aggragate later
                .load_and_count_pages::<(String, ValidatorDb, Option<BigDecimal>)>(
                    conn,
                )

        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_unbonds_by_address(
        &self,
        address: String,
        page: i64,
    ) -> Result<(Vec<(ValidatorDb, UnbondDb)>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .inner_join(unbonds::table)
                .filter(unbonds::dsl::address.eq(address))
                .select((validators::all_columns, unbonds::all_columns))
                .paginate(page)
                .load_and_count_pages::<(ValidatorDb, UnbondDb)>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_withdraws_by_address(
        &self,
        address: String,
        current_epoch: i32,
        page: i64,
    ) -> Result<(Vec<(ValidatorDb, UnbondDb)>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .inner_join(unbonds::table)
                .filter(
                    unbonds::dsl::address
                        .eq(address)
                        .and(unbonds::dsl::withdraw_epoch.le(current_epoch)),
                )
                .select((validators::all_columns, unbonds::all_columns))
                .paginate(page)
                .load_and_count_pages::<(ValidatorDb, UnbondDb)>(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_rewards_by_address(
        &self,
        address: String,
    ) -> Result<Vec<PoSRewardDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            pos_rewards::table
                .filter(pos_rewards::dsl::owner.eq(address))
                .select(PoSRewardDb::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn get_total_voting_power(&self) -> Result<Option<i64>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .select(sum(validators::dsl::voting_power))
                .first(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
