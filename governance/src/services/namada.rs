use anyhow::Context;
use diesel::ExpressionMethods;
use futures::StreamExt;
use namada_sdk::rpc;
use shared::block::Epoch;
use shared::proposal::{GovernanceProposalResult, GovernanceProposalStatus};
use shared::utils::GovernanceProposalShort;
use tendermint_rpc::HttpClient;

pub async fn query_last_epoch(client: &HttpClient) -> anyhow::Result<Epoch> {
    let epoch = rpc::query_epoch(client)
        .await
        .with_context(|| "Failed to query Namada's epoch epoch".to_string())?;
    Ok(epoch.0 as Epoch)
}

pub async fn get_governance_proposals_updates(
    client: &HttpClient,
    proposal_data: Vec<GovernanceProposalShort>,
    current_epoch: Epoch,
) -> anyhow::Result<Vec<GovernanceProposalStatus>> {
    let current_epoch = current_epoch as u64;

    Ok(futures::stream::iter(proposal_data)
        .filter_map(|proposal| async move {
            tracing::info!("Fetching proposal {} ...", proposal.id);
            let proposal_result =
                rpc::query_proposal_result(client, proposal.id).await;
            tracing::info!("Done fetching proposal {}!", proposal.id);

            if let Ok(Some(proposal_result)) = proposal_result {
                let result = if current_epoch.ge(&proposal.voting_end_epoch) {
                    match proposal_result.result {
                        namada_governance::utils::TallyResult::Passed => {
                            GovernanceProposalResult::Passed
                        }
                        namada_governance::utils::TallyResult::Rejected => {
                            GovernanceProposalResult::Rejected
                        }
                    }
                } else if current_epoch.ge(&proposal.voting_start_epoch)
                    && current_epoch.le(&proposal.voting_end_epoch)
                {
                    GovernanceProposalResult::VotingPeriod
                } else {
                    GovernanceProposalResult::Pending
                };

                Some(GovernanceProposalStatus {
                    id: proposal.id,
                    result,
                    yay_votes: proposal_result
                        .total_yay_power
                        .to_string_native(),
                    nay_votes: proposal_result
                        .total_nay_power
                        .to_string_native(),
                    abstain_votes: proposal_result
                        .total_abstain_power
                        .to_string_native(),
                })
            } else {
                None
            }
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await)
}
