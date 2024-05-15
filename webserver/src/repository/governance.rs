use axum::async_trait;
use orm::governance_proposal::{
    GovernanceProposalDb, GovernanceProposalResultDb,
};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct GovernanceRepo {
    pub(crate) app_state: AppState,
}

#[async_trait]
pub trait GovernanceRepoTrait {
    fn new(app_state: AppState) -> Self;

    async fn find_governance_proposals(
        &self,
        status: Option<Vec<GovernanceProposalResultDb>>,
        page: i32,
        item_per_page: i32,
    ) -> Result<Vec<GovernanceProposalDb>, String>;

    async fn find_governance_proposals_by_id(
        &self,
        proposal_id: u64,
    ) -> Result<Option<GovernanceProposalDb>, String>;
}

#[async_trait]
impl GovernanceRepoTrait for GovernanceRepo {
    fn new(app_state: AppState) -> Self {
        todo!()
    }

    async fn find_governance_proposals(
        &self,
        status: Option<Vec<GovernanceProposalResultDb>>,
        page: i32,
        item_per_page: i32,
    ) -> Result<Vec<GovernanceProposalDb>, String> {
        todo!()
    }

    async fn find_governance_proposals_by_id(
        &self,
        proposal_id: u64,
    ) -> Result<Option<GovernanceProposalDb>, String> {
        todo!()
    }
}
