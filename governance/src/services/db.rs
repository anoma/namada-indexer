use anyhow::{anyhow, Context};
use deadpool_diesel::postgres::Object;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use namada_core::storage::BlockHeight as NamadaSdkBlockHeight;
use namada_sdk::rpc;
use orm::{
    governance_proposal::GovernanceProposalResultDb,
    schema::governance_proposals,
};
use shared::{
    block::{BlockHeight, Epoch},
    error::ContextDbInteractError,
    utils::GovernanceProposalShort,
};
use tendermint_rpc::HttpClient;

pub async fn get_all_running_proposals(
    conn: Object,
    current_epoch: i32,
) -> anyhow::Result<Vec<GovernanceProposalShort>> {
    use diesel::connection::DefaultLoadingMode;

    conn.interact(move |conn| {
        anyhow::Ok(
            governance_proposals::table
                .filter(
                    governance_proposals::dsl::result
                        .ne(GovernanceProposalResultDb::Passed)
                        .and(
                            governance_proposals::dsl::result
                                .ne(GovernanceProposalResultDb::Rejected)
                                .and(
                                    governance_proposals::dsl::end_epoch
                                        .ge(current_epoch),
                                )
                                .and(
                                    governance_proposals::dsl::start_epoch
                                        .le(current_epoch),
                                ),
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
                    let (id, voting_start_epoch, voting_end_epoch) = result
                        .context("Failed to deserialize proposal from db")?;
                    anyhow::Ok(GovernanceProposalShort {
                        id: id as u64,
                        voting_start_epoch: voting_start_epoch as u64,
                        voting_end_epoch: voting_end_epoch as u64,
                    })
                })
                .collect::<Result<Vec<GovernanceProposalShort>, _>>()?,
        )
    })
    .await
    .context_db_interact_error()?
    .context("Failed to get governance proposals to be tallied from db")
}
