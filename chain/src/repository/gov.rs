use anyhow::Context;
use diesel::{PgConnection, RunQueryDsl};
use orm::{
    governance_proposal::GovernanceProposalInsertDb,
    governance_votes::GovernanceProposalVoteInsertDb,
};
use shared::{proposal::GovernanceProposal, vote::GovernanceVote};

pub fn insert_proposals(
    transaction_conn: &mut PgConnection,
    proposals: Vec<GovernanceProposal>,
) -> anyhow::Result<()> {
    diesel::insert_into(orm::schema::governance_proposals::table)
        .values::<&Vec<GovernanceProposalInsertDb>>(
            &proposals
                .into_iter()
                .map(|proposal| {
                    GovernanceProposalInsertDb::from_governance_proposal(
                        proposal,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to update governance proposals in db")?;

    anyhow::Ok(())
}

pub fn insert_votes(
    transaction_conn: &mut PgConnection,
    proposals_votes: Vec<GovernanceVote>,
) -> anyhow::Result<()> {
    diesel::insert_into(orm::schema::governance_votes::table)
        .values::<&Vec<GovernanceProposalVoteInsertDb>>(
            &proposals_votes
                .into_iter()
                .map(|vote| {
                    GovernanceProposalVoteInsertDb::from_governance_vote(vote)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to update governance votes in db")?;

    anyhow::Ok(())
}
