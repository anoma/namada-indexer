use std::{collections::HashSet, str::FromStr};

use anyhow::{anyhow, Context};
use namada_core::storage::BlockHeight as NamadaSdkBlockHeight;
use namada_sdk::{
    address::Address,
    queries::RPC,
    rpc::{self, query_storage_value},
};
use namada_token as token;
use tendermint_rpc::HttpClient;

use shared::{
    balance::{Amount, Balance, Balances},
    block::{BlockHeight, Epoch},
    id::Id,
    utils::BalanceChange,
};

pub async fn is_block_committed(
    client: &HttpClient,
    block_height: BlockHeight,
) -> anyhow::Result<bool> {
    let block_height = to_block_height(block_height);
    let last_block = RPC
        .shell()
        .last_block(client)
        .await
        .context("Failed to query Namada's last committed block")?;
    Ok(last_block
        .map(|b| block_height <= b.height)
        .unwrap_or(false))
}

pub async fn get_native_token(client: &HttpClient) -> anyhow::Result<Id> {
    let native_token = RPC
        .shell()
        .native_token(client)
        .await
        .context("Failed to query native token")?;
    Ok(Id::from(native_token))
}

pub async fn get_epoch_at_block_height(
    client: &HttpClient,
    block_height: BlockHeight,
) -> anyhow::Result<Epoch> {
    let block_height = to_block_height(block_height);
    let epoch = rpc::query_epoch_at_height(client, block_height)
        .await
        .with_context(|| {
            format!("Failed to query Namada's epoch at height {block_height}")
        })?
        .ok_or_else(|| {
            anyhow!("No Namada epoch found for height {block_height}")
        })?;
    Ok(epoch.0 as Epoch)
}

pub async fn query_balance(
    client: &HttpClient,
    balance_changes: &HashSet<BalanceChange>,
) -> anyhow::Result<Balances> {
    let mut res: Balances = vec![];

    for balance_change in balance_changes {
        let owner =
            Address::from_str(&balance_change.address.to_string()).unwrap();
        let token =
            Address::from_str(&balance_change.token.to_string()).unwrap();

        let amount = rpc::get_token_balance(client, &token, &owner).await.unwrap_or_default();

        res.push(Balance {
            owner: owner.to_string(),
            token: token.to_string(),
            amount: Amount::from(amount),
        });
    }

    anyhow::Ok(res)
}

fn to_block_height(block_height: u32) -> NamadaSdkBlockHeight {
    NamadaSdkBlockHeight::from(block_height as u64)
}
