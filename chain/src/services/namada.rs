use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use namada_core::storage::{
    BlockHeight as NamadaSdkBlockHeight, Epoch as NamadaSdkEpoch,
};
use namada_sdk::address::Address as NamadaSdkAddress;
use namada_sdk::queries::RPC;
use namada_sdk::rpc;
use shared::balance::{Amount, Balance, Balances};
use shared::block::{BlockHeight, Epoch};
use shared::id::Id;
use shared::unbond::{Unbond, UnbondAddresses, Unbonds};
use shared::utils::BalanceChange;
use tendermint_rpc::HttpClient;

use shared::bond::{Bond, BondAddresses, Bonds};

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

// TODO: remove unwraps
pub async fn query_balance(
    client: &HttpClient,
    balance_changes: &HashSet<BalanceChange>,
) -> anyhow::Result<Balances> {
    let mut res: Balances = vec![];

    for balance_change in balance_changes {
        let owner =
            NamadaSdkAddress::from_str(&balance_change.address.to_string())
                .unwrap();
        let token =
            NamadaSdkAddress::from_str(&balance_change.token.to_string())
                .unwrap();

        let amount = rpc::get_token_balance(client, &token, &owner)
            .await
            .unwrap_or_default();

        res.push(Balance {
            owner: Id::from(owner),
            token: Id::from(token),
            amount: Amount::from(amount),
        });
    }

    anyhow::Ok(res)
}

pub async fn query_next_governance_id(
    client: &HttpClient,
    block_height: BlockHeight,
) -> anyhow::Result<u64> {
    let proposal_counter_key =
        namada_sdk::governance::storage::keys::get_counter_key();
    let block_height = to_block_height(block_height);

    let query_result = RPC
        .shell()
        .storage_value(
            client,
            None,
            Some(block_height),
            false,
            &proposal_counter_key,
        )
        .await
        .context("Failed to get the next proposal id")?;
    namada_sdk::borsh::BorshDeserialize::try_from_slice(&query_result.data)
        .context("Failed to deserialize proposal id")
}

pub async fn query_bonds(
    client: &HttpClient,
    addresses: Vec<BondAddresses>,
    epoch: Epoch,
) -> anyhow::Result<Bonds> {
    let mut bonds = vec![];

    for BondAddresses { source, target } in addresses {
        //TODO: unwrap
        let source = NamadaSdkAddress::from_str(&source.to_string()).unwrap();
        let target = NamadaSdkAddress::from_str(&target.to_string()).unwrap();

        //TODO: should we get bond with slashes
        let amount =
            rpc::query_bond(client, &source, &target, Some(to_epoch(epoch)))
                .await
                .context("Failed to query bond amount")?;

        bonds.push(Bond {
            source: Id::from(source),
            target: Id::from(target),
            amount: Amount::from(amount),
        });
    }

    anyhow::Ok(Bonds {
        epoch,
        values: bonds,
    })
}

pub async fn query_unbonds(
    client: &HttpClient,
    addresses: Vec<UnbondAddresses>,
) -> anyhow::Result<Unbonds> {
    let mut unbonds = vec![];

    for UnbondAddresses { source, target } in addresses {
        //TODO: unwrap
        let source = NamadaSdkAddress::from_str(&source.to_string()).unwrap();
        let target = NamadaSdkAddress::from_str(&target.to_string()).unwrap();

        let res = rpc::query_unbond_with_slashing(client, &source, &target)
            .await
            .context("Failed to query unbond amount")?;

        let ((_, withdraw_epoch), amount) =
            res.last().context("Unbonds are empty")?;

        unbonds.push(Unbond {
            source: Id::from(source),
            target: Id::from(target),
            amount: Amount::from(*amount),
            withdraw_at: withdraw_epoch.0 as Epoch,
        });
    }

    //TODO: remove epoch
    anyhow::Ok(unbonds)
}

fn to_block_height(block_height: u32) -> NamadaSdkBlockHeight {
    NamadaSdkBlockHeight::from(block_height as u64)
}

fn to_epoch(epoch: u32) -> NamadaSdkEpoch {
    NamadaSdkEpoch::from(epoch as u64)
}
