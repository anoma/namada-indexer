use anyhow::Context;
use diesel::connection::DefaultLoadingMode;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl,
};
use orm::governance_proposal::{
    GovernanceProposalResultDb, GovernanceProposalUpdateStatusDb,
};
use orm::schema::governance_proposals;
use shared::utils::GovernanceProposalShort;

pub fn get_all_running_proposals(
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<GovernanceProposalShort>> {
    governance_proposals::table
        .filter(
            governance_proposals::dsl::result
                .ne(GovernanceProposalResultDb::Passed)
                .and(
                    governance_proposals::dsl::result
                        .ne(GovernanceProposalResultDb::Rejected),
                ),
        )
        .select((
            governance_proposals::dsl::id,
            governance_proposals::dsl::start_epoch,
            governance_proposals::dsl::end_epoch,
        ))
        .load_iter::<(i32, i32, i32), DefaultLoadingMode>(conn)
        .context("Failed to get governance proposal ids from db")?
        .map(|result| {
            let (id, voting_start_epoch, voting_end_epoch) =
                result.context("Failed to deserialize proposal from db")?;
            anyhow::Ok(GovernanceProposalShort {
                id: id as u64,
                voting_start_epoch: voting_start_epoch as u64,
                voting_end_epoch: voting_end_epoch as u64,
            })
        })
        .collect::<Result<Vec<GovernanceProposalShort>, _>>()
}

pub fn update_proposal_status(
    transaction_conn: &mut PgConnection,
    proposal_id: u64,
    proposal_status: GovernanceProposalUpdateStatusDb,
) -> anyhow::Result<()> {
    diesel::update(governance_proposals::table.find(proposal_id as i32))
        .set::<GovernanceProposalUpdateStatusDb>(proposal_status)
        .execute(transaction_conn)?;

    Ok(())
}
