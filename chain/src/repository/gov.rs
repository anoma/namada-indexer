use std::collections::HashSet;

use anyhow::Context;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::governance_proposal::GovernanceProposalInsertDb;
use orm::governance_votes::GovernanceProposalVoteInsertDb;
use orm::schema::{governance_proposals, governance_votes};
use shared::proposal::{GovernanceProposal, TallyType};
use shared::tuple_len::TupleLen;
use shared::vote::GovernanceVote;

use super::utils::MAX_PARAM_SIZE;

pub fn insert_proposals(
    transaction_conn: &mut PgConnection,
    proposals: Vec<(GovernanceProposal, TallyType)>,
) -> anyhow::Result<()> {
    diesel::insert_into(governance_proposals::table)
        .values::<&Vec<GovernanceProposalInsertDb>>(
            &proposals
                .into_iter()
                .map(|(proposal, tally_type)| {
                    GovernanceProposalInsertDb::from_governance_proposal(
                        proposal, tally_type,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict(governance_proposals::id)
        .do_update()
        .set((
            governance_proposals::result
                .eq(excluded(governance_proposals::result)),
            governance_proposals::yay_votes
                .eq(excluded(governance_proposals::yay_votes)),
            governance_proposals::nay_votes
                .eq(excluded(governance_proposals::nay_votes)),
            governance_proposals::abstain_votes
                .eq(excluded(governance_proposals::abstain_votes)),
        ))
        .execute(transaction_conn)
        .context("Failed to update governance proposals in db")?;

    anyhow::Ok(())
}

pub fn insert_votes(
    transaction_conn: &mut PgConnection,
    proposals_votes: HashSet<GovernanceVote>,
) -> anyhow::Result<()> {
    let votes_col_count = governance_votes::all_columns.len() as i64;

    for chunk in proposals_votes
        .into_iter()
        .collect::<Vec<_>>()
        .chunks((MAX_PARAM_SIZE as i64 / votes_col_count) as usize)
    {
        insert_votes_chunk(transaction_conn, chunk.to_vec())?
    }

    anyhow::Ok(())
}

fn insert_votes_chunk(
    transaction_conn: &mut PgConnection,
    proposals_votes: Vec<GovernanceVote>,
) -> anyhow::Result<()> {
    diesel::insert_into(governance_votes::table)
        .values::<&Vec<GovernanceProposalVoteInsertDb>>(
            &proposals_votes
                .into_iter()
                .map(|vote| {
                    GovernanceProposalVoteInsertDb::from_governance_vote(vote)
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            governance_votes::voter_address,
            governance_votes::proposal_id,
        ))
        .do_update()
        .set((governance_votes::kind.eq(excluded(governance_votes::kind)),))
        .execute(transaction_conn)
        .context("Failed to update governance votes in db")?;

    anyhow::Ok(())
}
