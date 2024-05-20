use orm::governance_proposal::GovernanceProposalResultDb;

use crate::appstate::AppState;
use crate::dto::governance::ProposalStatus;
use crate::error::governance::GovernanceError;
use crate::repository::governance::{GovernanceRepo, GovernanceRepoTrait};
use crate::response::governance::{Proposal, ProposalVote};

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
        page: u64,
    ) -> Result<(Vec<Proposal>, u64), GovernanceError> {
        let status = status
            .map(|s| match s {
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
            })
            .unwrap_or_else(|| {
                vec![
                    GovernanceProposalResultDb::Pending,
                    GovernanceProposalResultDb::VotingPeriod,
                    GovernanceProposalResultDb::Passed,
                    GovernanceProposalResultDb::Rejected,
                ]
            });

        let (db_proposals, total_items) = self
            .governance_repo
            .find_governance_proposals(status, page as i64)
            .await
            .map_err(GovernanceError::Database)?;

        Ok((
            db_proposals.into_iter().map(Proposal::from).collect(),
            total_items as u64,
        ))
    }

    pub async fn find_governance_proposal_by_id(
        &self,
        proposal_id: u64,
    ) -> Result<Option<Proposal>, GovernanceError> {
        let db_proposal = self
            .governance_repo
            .find_governance_proposals_by_id(proposal_id as i32)
            .await
            .map_err(GovernanceError::Database)?;

        Ok(db_proposal.map(Proposal::from))
    }

    pub async fn search_governance_proposals_by_pattern(
        &self,
        pattern: String,
        page: u64,
    ) -> Result<(Vec<Proposal>, u64), GovernanceError> {
        if pattern.len() < 3 {
            return Err(GovernanceError::TooShortPattern(pattern.len()));
        }

        let (db_proposals, total_items) = self
            .governance_repo
            .search_governance_proposals_by_pattern(pattern, page as i64)
            .await
            .map_err(GovernanceError::Database)?;

        Ok((
            db_proposals.into_iter().map(Proposal::from).collect(),
            total_items as u64,
        ))
    }

    pub async fn find_governance_proposal_votes(
        &self,
        proposal_id: u64,
        page: u64,
    ) -> Result<(Vec<ProposalVote>, u64), GovernanceError> {
        let (db_proposal_votes, total_items) = self
            .governance_repo
            .find_governance_proposal_votes(proposal_id as i32, page as i64)
            .await
            .map_err(GovernanceError::Database)?;

        Ok((
            db_proposal_votes
                .into_iter()
                .map(ProposalVote::from)
                .collect(),
            total_items as u64,
        ))
    }

    pub async fn find_governance_proposal_votes_by_address(
        &self,
        proposal_id: u64,
        voter_address: String,
    ) -> Result<Vec<ProposalVote>, GovernanceError> {
        let db_proposal_votes = self
            .governance_repo
            .find_governance_proposal_votes_by_address(
                proposal_id as i32,
                voter_address,
            )
            .await
            .map_err(GovernanceError::Database)?;

        Ok(db_proposal_votes
            .into_iter()
            .map(ProposalVote::from)
            .collect())
    }
}
