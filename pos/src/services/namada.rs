use anyhow::{anyhow, Context};
use futures::{StreamExt, TryStreamExt};
use namada_core::storage::Epoch as NamadaSdkEpoch;
use namada_sdk::rpc;
use shared::block::Epoch;
use shared::id::Id;
use shared::validator::{Validator, ValidatorSet};
use tendermint_rpc::HttpClient;

pub async fn get_validator_set_at_epoch(
    client: &HttpClient,
    epoch: Epoch,
) -> anyhow::Result<ValidatorSet> {
    let namada_epoch = to_epoch(epoch);
    let validator_set = rpc::get_all_validators(client, namada_epoch)
        .await
        .with_context(|| {
            format!(
                "Failed to query Namada's consensus validators at epoch \
                 {epoch}"
            )
        })?;

    let validators = futures::stream::iter(validator_set)
        .map(|address| async move {
            let voting_power_fut = async {
                rpc::get_validator_stake(client, namada_epoch, &address)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to query the stake of validator {address} \
                             at epoch {namada_epoch}"
                        )
                    })
            };

            let meta_and_comm_fut = async {
                let (metadata, commission) =
                    rpc::query_metadata(client, &address, Some(namada_epoch))
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to query metadata of validator \
                                 {address} at epoch {namada_epoch}"
                            )
                        })?;
                metadata.zip(commission).ok_or_else(|| {
                    anyhow!(
                        "Metadata and commission must be present for \
                         validator {address} at epoch {namada_epoch}"
                    )
                })
            };

            let (voting_power, (metadata, commission)) =
                futures::try_join!(voting_power_fut, meta_and_comm_fut,)?;

            anyhow::Ok(Validator {
                address: Id::Account(address.to_string()),
                voting_power: voting_power.to_string_native(),
                max_commission: commission
                    .max_commission_change_per_epoch
                    .to_string(),
                commission: commission.commission_rate.to_string(),
                email: metadata.email,
                description: metadata.description,
                website: metadata.website,
                discord_handler: metadata.discord_handle,
                avatar: metadata.avatar,
            })
        })
        .buffer_unordered(100)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(ValidatorSet { validators, epoch })
}

pub async fn get_current_epoch(client: &HttpClient) -> anyhow::Result<Epoch> {
    let epoch = rpc::query_epoch(client)
        .await
        .context("Failed to query Namada's current epoch")?;

    Ok(epoch.0 as Epoch)
}

fn to_epoch(epoch: u32) -> NamadaSdkEpoch {
    NamadaSdkEpoch::from(epoch as u64)
}
