use axum::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use orm::bond::BondDb;
use orm::pos_rewards::PoSRewardDb;
use orm::schema::{bonds, pos_rewards, unbonds, validators};
use orm::unbond::UnbondDb;
use orm::validators::ValidatorDb;

use super::utils::Paginate;
use crate::appstate::AppState;

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
    ) -> Result<(Vec<ValidatorDb>, i64), String>;

    async fn find_validator_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ValidatorDb>, String>;

    async fn find_bonds_by_address(
        &self,
        address: String,
    ) -> Result<Vec<BondDb>, String>;

    async fn find_unbonds_by_address(
        &self,
        address: String,
    ) -> Result<Vec<UnbondDb>, String>;

    async fn find_withdraws_by_address(
        &self,
        address: String,
        current_epoch: i32,
    ) -> Result<Vec<UnbondDb>, String>;

    async fn find_rewards_by_address(
        &self,
        address: String,
    ) -> Result<Vec<PoSRewardDb>, String>;
}

#[async_trait]
impl PosRepositoryTrait for PosRepository {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_all_validators(
        &self,
        page: i64,
    ) -> Result<(Vec<ValidatorDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            validators::table
                .select(ValidatorDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn)
                .unwrap()
        })
        .await
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
    ) -> Result<Vec<BondDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            bonds::table
                .filter(bonds::dsl::address.eq(address))
                .select(BondDb::as_select())
                .get_results(conn)
                .unwrap()
        })
        .await
        .map_err(|e| e.to_string())
    }

    async fn find_unbonds_by_address(
        &self,
        address: String,
    ) -> Result<Vec<UnbondDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            unbonds::table
                .filter(unbonds::dsl::address.eq(address))
                .select(UnbondDb::as_select())
                .get_results(conn)
                .unwrap()
        })
        .await
        .map_err(|e| e.to_string())
    }

    async fn find_withdraws_by_address(
        &self,
        address: String,
        current_epoch: i32,
    ) -> Result<Vec<UnbondDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            unbonds::table
                .filter(
                    unbonds::dsl::address
                        .eq(address)
                        .and(unbonds::dsl::withdraw_epoch.ge(current_epoch)),
                )
                .select(UnbondDb::as_select())
                .get_results(conn)
                .unwrap()
        })
        .await
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
                .unwrap()
        })
        .await
        .map_err(|e| e.to_string())
    }
}
