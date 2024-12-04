use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::queries::RPC;
use namada_sdk::storage::{self, PrefixValue};
use shared::block::BlockHeight;
use tendermint_rpc::HttpClient;

/// Query a range of storage values with a matching prefix and decode them with
/// [`BorshDeserialize`]. Returns an iterator of the storage keys paired with
/// their associated values.
pub async fn query_storage_prefix<T>(
    client: &HttpClient,
    key: &storage::Key,
    height: Option<BlockHeight>,
) -> anyhow::Result<Option<impl Iterator<Item = (storage::Key, T)>>>
where
    T: BorshDeserialize,
{
    let values = RPC
        .shell()
        .storage_prefix(
            client,
            None,
            height.map(super::namada::to_block_height),
            false,
            key,
        )
        .await?;

    let decode = |PrefixValue { key, value }: PrefixValue| {
        T::try_from_slice(&value[..]).map(|value| (key, value)).ok()
    };

    Ok(if values.data.is_empty() {
        None
    } else {
        Some(values.data.into_iter().filter_map(decode))
    })
}
