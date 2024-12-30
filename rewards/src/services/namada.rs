use std::collections::HashSet;

use anyhow::Context;
use futures::StreamExt;
use namada_sdk::queries::RPC;
use namada_sdk::rpc;
use shared::balance::Amount;
use shared::block::Epoch;
use shared::id::Id;
use shared::rewards::Reward;
use shared::utils::DelegationPair;
use tendermint_rpc::HttpClient;

pub async fn query_delegation_pairs(
    client: &HttpClient,
) -> anyhow::Result<HashSet<DelegationPair>> {
    let data = rpc::bonds_and_unbonds(client, &None, &None)
        .await
        .with_context(|| {
            "Failed to query Namada's bonds and unbonds".to_string()
        })?;

    let pairs =
        data.into_iter()
            .fold(HashSet::new(), |mut acc, (bond_id, _)| {
                acc.insert(DelegationPair {
                    validator_address: Id::from(bond_id.validator),
                    delegator_address: Id::from(bond_id.source),
                });
                acc
            });

    anyhow::Ok(pairs)
}

pub async fn query_rewards(
    client: &HttpClient,
    delegation_pairs: HashSet<DelegationPair>,
) -> anyhow::Result<Vec<Reward>> {
    let epoch = get_current_epoch(client).await?;

    Ok(futures::stream::iter(delegation_pairs)
        .filter_map(|delegation| async move {
            tracing::info!(
                "Fetching rewards {} -> {} ...",
                delegation.validator_address,
                delegation.delegator_address
            );
            let reward = RPC
                .vp()
                .pos()
                .rewards(
                    client,
                    &delegation.validator_address.clone().into(),
                    &Some(delegation.delegator_address.clone().into()),
                )
                .await
                .ok()?;

            tracing::info!(
                "Done fetching reward for {} -> {}!",
                delegation.validator_address,
                delegation.delegator_address
            );

            Some(Reward {
                delegation_pair: delegation,
                amount: Amount::from(reward),
                epoch: epoch as i32,
            })
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await)
}

pub async fn get_current_epoch(client: &HttpClient) -> anyhow::Result<Epoch> {
    let epoch = rpc::query_epoch(client)
        .await
        .context("Failed to query Namada's current epoch")?;

    Ok(epoch.0 as Epoch)
}
