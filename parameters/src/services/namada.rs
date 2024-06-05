use anyhow::Context;
use namada_parameters::EpochDuration;
use namada_sdk::rpc;
use shared::block::Epoch;
use shared::parameters::Parameters;
use tendermint_rpc::HttpClient;

pub async fn get_parameters(
    client: &HttpClient,
    epoch: Epoch,
) -> anyhow::Result<Parameters> {
    let pos_parameters = rpc::get_pos_params(client)
        .await
        .with_context(|| "Failed to query pos parameters".to_string())?;

    let epochs_per_year_key =
        namada_parameters::storage::get_epochs_per_year_key();
    let epochs_per_year: u64 =
        rpc::query_storage_value(client, &epochs_per_year_key)
            .await
            .with_context(|| {
                "Failed to query epochs_per_year parameter".to_string()
            })?;

    let epoch_duration_key =
        namada_parameters::storage::get_epoch_duration_storage_key();
    let epoch_duration: EpochDuration =
        rpc::query_storage_value(client, &epoch_duration_key)
            .await
            .with_context(|| {
                "Failed to query epochs_per_year parameter".to_string()
            })?;

    Ok(Parameters {
        epoch,
        unbonding_length: pos_parameters.unbonding_len,
        pipeline_length: pos_parameters.pipeline_len,
        epochs_per_year,
        min_num_of_blocks: epoch_duration.min_num_of_blocks,
        min_duration: epoch_duration.min_duration.0,
    })
}

pub async fn get_current_epoch(client: &HttpClient) -> anyhow::Result<Epoch> {
    let epoch = rpc::query_epoch(client)
        .await
        .context("Failed to query Namada's current epoch")?;

    Ok(epoch.0 as Epoch)
}
