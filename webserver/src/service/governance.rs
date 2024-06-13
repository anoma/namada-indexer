use orm::governance_proposal::{
    GovernanceProposalKindDb, GovernanceProposalResultDb,
};

use crate::appstate::AppState;
use crate::dto::governance::{ProposalKind, ProposalStatus};
use crate::error::governance::GovernanceError;
use crate::repository::chain::{ChainRepository, ChainRepositoryTrait};
use crate::repository::governance::{GovernanceRepo, GovernanceRepoTrait};
use crate::response::governance::{Proposal, ProposalVote};

#[derive(Clone)]
pub struct GovernanceService {
    governance_repo: GovernanceRepo,
    chain_repo: ChainRepository,
}

impl GovernanceService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            governance_repo: GovernanceRepo::new(app_state.clone()),
            chain_repo: ChainRepository::new(app_state),
        }
    }

    pub async fn find_governance_proposals(
        &self,
        status: Option<ProposalStatus>,
        kind: Option<ProposalKind>,
        pattern: Option<String>,
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
                ProposalStatus::Passed => {
                    vec![GovernanceProposalResultDb::Passed]
                }
                ProposalStatus::Rejected => {
                    vec![GovernanceProposalResultDb::Rejected]
                }
            })
            .unwrap_or_else(|| {
                vec![
                    GovernanceProposalResultDb::Pending,
                    GovernanceProposalResultDb::VotingPeriod,
                    GovernanceProposalResultDb::Passed,
                    GovernanceProposalResultDb::Rejected,
                ]
            });

        let kind = kind.map(|t| match t {
            ProposalKind::Default => GovernanceProposalKindDb::Default,
            ProposalKind::DefaultWithWasm => {
                GovernanceProposalKindDb::DefaultWithWasm
            }
            ProposalKind::PgfSteward => GovernanceProposalKindDb::PgfSteward,
            ProposalKind::PgfFunding => GovernanceProposalKindDb::PgfFunding,
        });

        let (db_proposals, total_items) = self
            .governance_repo
            .find_governance_proposals(status, kind, pattern, page as i64)
            .await
            .map_err(GovernanceError::Database)?;

        let chain_state = self
            .chain_repo
            .get_chain_state()
            .await
            .map_err(GovernanceError::Database)?;

        let parameters = self
            .chain_repo
            .find_chain_parameters()
            .await
            .map_err(GovernanceError::Database)?;

        Ok((
            db_proposals
                .into_iter()
                .map(|p| {
                    Proposal::from_proposal_db(
                        p,
                        &chain_state,
                        parameters.min_num_of_blocks,
                        parameters.min_duration,
                    )
                })
                .collect(),
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

        let chain_state = self
            .chain_repo
            .get_chain_state()
            .await
            .map_err(GovernanceError::Database)?;

        let parameters = self
            .chain_repo
            .find_chain_parameters()
            .await
            .map_err(GovernanceError::Database)?;

        Ok(db_proposal.map(|p| {
            Proposal::from_proposal_db(
                p,
                &chain_state,
                parameters.min_num_of_blocks,
                parameters.min_duration,
            )
        }))
    }

    pub async fn find_governance_proposal_votes(
        &self,
        proposal_id: u64,
        page: u64,
    ) -> Result<(Vec<ProposalVote>, u64), GovernanceError> {
        let db_proposal = self
            .governance_repo
            .find_governance_proposals_by_id(proposal_id as i32)
            .await
            .map_err(GovernanceError::Database)?;

        if db_proposal.is_none() {
            return Err(GovernanceError::NotFound(proposal_id));
        }

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
        let db_proposal = self
            .governance_repo
            .find_governance_proposals_by_id(proposal_id as i32)
            .await
            .map_err(GovernanceError::Database)?;

        if db_proposal.is_none() {
            return Err(GovernanceError::NotFound(proposal_id));
        }

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

    pub async fn find_governance_proposal_votes_by_voter(
        &self,
        voter_address: String,
    ) -> Result<Vec<ProposalVote>, GovernanceError> {
        let db_proposal_votes = self
            .governance_repo
            .find_governance_proposal_votes_by_voter(voter_address)
            .await
            .map_err(GovernanceError::Database)?;

        Ok(db_proposal_votes
            .into_iter()
            .map(ProposalVote::from)
            .collect())
    }
}
