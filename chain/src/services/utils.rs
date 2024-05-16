use namada_sdk::{
    borsh::BorshDeserialize,
    queries::RPC,
    storage::{self, PrefixValue},
};
use tendermint_rpc::HttpClient;

/// Query a range of storage values with a matching prefix and decode them with
/// [`BorshDeserialize`]. Returns an iterator of the storage keys paired with
/// their associated values.
pub async fn query_storage_prefix<T>(
    client: &HttpClient,
    key: &storage::Key,
) -> anyhow::Result<Option<impl Iterator<Item = (storage::Key, T)>>>
where
    T: BorshDeserialize,
{
    let values = RPC
        .shell()
        .storage_prefix(client, None, None, false, key)
        .await?;

    let decode =
        |PrefixValue { key, value }: PrefixValue| match T::try_from_slice(
            &value[..],
        ) {
            //TODO: do sth with err
            Err(_err) => None,
            Ok(value) => Some((key, value)),
        };
    Ok(if values.data.is_empty() {
        None
    } else {
        Some(values.data.into_iter().filter_map(decode))
    })
}
