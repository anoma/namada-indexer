use orm::governance_proposal::GovernanceProposalResultDb;

use crate::appstate::AppState;
use crate::constant::ITEM_PER_PAGE;
use crate::dto::governance::ProposalStatus;
use crate::dto::utils::Pagination;
use crate::error::governance::GovernanceError;
use crate::repository::governance::{GovernanceRepo, GovernanceRepoTrait};
use crate::response::governance::Proposal;

#[derive(Clone)]
pub struct GovernanceService {
    governance_repo: GovernanceRepo,
}

impl GovernanceService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            governance_repo: GovernanceRepo::new(app_state),
        }
    }

    pub async fn find_governance_proposals(
        &self,
        status: Option<ProposalStatus>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<Proposal>, GovernanceError> {
        let page = pagination.map(|p| p.page).unwrap_or(0);
        let status = status.map(|s| match s {
            ProposalStatus::Pending => {
                vec![GovernanceProposalResultDb::Pending]
            }
            ProposalStatus::VotingPeriod => {
                vec![GovernanceProposalResultDb::VotingPeriod]
            }
            ProposalStatus::Ended => vec![
                GovernanceProposalResultDb::Passed,
                GovernanceProposalResultDb::Rejected,
            ],
        });

        let db_proposals = self
            .governance_repo
            .find_governance_proposals(
                status,
                page as i32,
                ITEM_PER_PAGE as i32,
            )
            .await
            .map_err(GovernanceError::Database)?;

        Ok(db_proposals.into_iter().map(Proposal::from).collect())
    }

    pub async fn find_governance_proposal_by_id(
        &self,
        proposal_id: u64,
    ) -> Result<Option<Proposal>, GovernanceError> {
        let db_proposal = self
            .governance_repo
            .find_governance_proposals_by_id(proposal_id)
            .await
            .map_err(GovernanceError::Database)?;

        Ok(db_proposal.map(Proposal::from))
    }
}
