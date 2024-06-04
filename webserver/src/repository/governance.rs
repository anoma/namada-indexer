use axum::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods,
    QueryDsl, RunQueryDsl, SelectableHelper,
};
use orm::governance_proposal::{
    GovernanceProposalDb, GovernanceProposalResultDb,
};
use orm::governance_votes::GovernanceProposalVoteDb;
use orm::schema::{governance_proposals, governance_votes};

use crate::appstate::AppState;
use crate::repository::utils::Paginate;

#[derive(Clone)]
pub struct GovernanceRepo {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait GovernanceRepoTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_governance_proposals(
        &self,
        status: Vec<GovernanceProposalResultDb>,
        page: i64,
    ) -> Result<(Vec<GovernanceProposalDb>, i64), String>;

    async fn find_governance_proposals_by_id(
        &self,
        proposal_id: i32,
    ) -> Result<Option<GovernanceProposalDb>, String>;

    async fn search_governance_proposals_by_pattern(
        &self,
        pattern: String,
        page: i64,
    ) -> Result<(Vec<GovernanceProposalDb>, i64), String>;

    async fn find_governance_proposal_votes(
        &self,
        proposal_id: i32,
        page: i64,
    ) -> Result<(Vec<GovernanceProposalVoteDb>, i64), String>;

    async fn find_governance_proposal_votes_by_address(
        &self,
        proposal_id: i32,
        voter_address: String,
    ) -> Result<Vec<GovernanceProposalVoteDb>, String>;

    async fn find_governance_proposal_votes_by_voter(
        &self,
        voter_address: String,
    ) -> Result<Vec<GovernanceProposalVoteDb>, String>;
}

#[async_trait]
impl GovernanceRepoTrait for GovernanceRepo {
    fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn find_governance_proposals(
        &self,
        status: Vec<GovernanceProposalResultDb>,
        page: i64,
    ) -> Result<(Vec<GovernanceProposalDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            governance_proposals::table
                .filter(governance_proposals::dsl::result.eq_any(status))
                .select(GovernanceProposalDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_governance_proposals_by_id(
        &self,
        proposal_id: i32,
    ) -> Result<Option<GovernanceProposalDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            governance_proposals::table
                .find(proposal_id)
                .select(GovernanceProposalDb::as_select())
                .first(conn)
                .ok()
        })
        .await
        .map_err(|e| e.to_string())
    }

    async fn search_governance_proposals_by_pattern(
        &self,
        pattern: String,
        page: i64,
    ) -> Result<(Vec<GovernanceProposalDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            governance_proposals::table
                .filter(
                    governance_proposals::dsl::content
                        .ilike(format!("%{}%", pattern)),
                )
                .select(GovernanceProposalDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_governance_proposal_votes(
        &self,
        proposal_id: i32,
        page: i64,
    ) -> Result<(Vec<GovernanceProposalVoteDb>, i64), String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            governance_votes::table
                .filter(governance_votes::dsl::id.eq(proposal_id))
                .select(GovernanceProposalVoteDb::as_select())
                .paginate(page)
                .load_and_count_pages(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_governance_proposal_votes_by_address(
        &self,
        proposal_id: i32,
        voter_address: String,
    ) -> Result<Vec<GovernanceProposalVoteDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            governance_votes::table
                .filter(governance_votes::dsl::id.eq(proposal_id).and(
                    governance_votes::dsl::voter_address.eq(voter_address),
                ))
                .select(GovernanceProposalVoteDb::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }

    async fn find_governance_proposal_votes_by_voter(
        &self,
        voter_address: String,
    ) -> Result<Vec<GovernanceProposalVoteDb>, String> {
        let conn = self.app_state.get_db_connection().await;

        conn.interact(move |conn| {
            governance_votes::table
                .filter(governance_votes::dsl::voter_address.eq(voter_address))
                .select(GovernanceProposalVoteDb::as_select())
                .get_results(conn)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
}
