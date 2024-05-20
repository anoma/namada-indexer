use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use futures::StreamExt;
use namada_core::storage::{
    BlockHeight as NamadaSdkBlockHeight, Epoch as NamadaSdkEpoch,
};
use namada_sdk::address::Address as NamadaSdkAddress;
use namada_sdk::collections::HashMap;
use namada_sdk::hash::Hash;
use namada_sdk::queries::RPC;
use namada_sdk::rpc::{
    bonds_and_unbonds, query_proposal_by_id, query_storage_value,
};
use namada_sdk::state::Key;
use namada_sdk::token::Amount as NamadaSdkAmount;
use namada_sdk::{borsh, rpc, token};
use shared::balance::{Amount, Balance, Balances};
use shared::block::{BlockHeight, Epoch};
use shared::bond::{Bond, BondAddresses, Bonds};
use shared::id::Id;
use shared::proposal::GovernanceProposal;
use shared::unbond::{Unbond, UnbondAddresses, Unbonds};
use shared::utils::BalanceChange;
use subtle_encoding::hex;
use tendermint_rpc::HttpClient;

use super::utils::query_storage_prefix;

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
    Ok(futures::stream::iter(balance_changes)
        .filter_map(|balance_change| async move {
            tracing::info!(
                "Fetching balance change for {} ...",
                balance_change.address
            );

            let owner =
                NamadaSdkAddress::from_str(&balance_change.address.to_string())
                    .context("Failed to parse owner address")
                    .ok()?;
            let token =
                NamadaSdkAddress::from_str(&balance_change.token.to_string())
                    .context("Failed to parse token address")
                    .ok()?;

            let amount = rpc::get_token_balance(client, &token, &owner)
                .await
                .unwrap_or_default();

            Some(Balance {
                owner: Id::from(owner),
                token: Id::from(token),
                amount: Amount::from(amount),
            })
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await)
}

pub async fn query_all_balances(
    client: &HttpClient,
) -> anyhow::Result<Balances> {
    let token_addr = RPC
        .shell()
        .native_token(client)
        .await
        .context("Failed to query native token")?;

    let balance_prefix = namada_token::storage_key::balance_prefix(&token_addr);

    let balances =
        query_storage_prefix::<token::Amount>(client, &balance_prefix)
            .await
            // TODO: unwrap
            .unwrap();

    let mut all_balances: Balances = vec![];

    if let Some(balances) = balances {
        for (key, balance) in balances {
            let (t, o, b) =
                match namada_token::storage_key::is_any_token_balance_key(&key)
                {
                    Some([tok, owner]) => (tok.clone(), owner.clone(), balance),
                    None => continue,
                };

            all_balances.push(Balance {
                owner: Id::from(o),
                token: Id::from(t),
                amount: Amount::from(b),
            });
        }
    }

    anyhow::Ok(all_balances)
}

pub async fn query_last_block_height(
    client: &HttpClient,
) -> anyhow::Result<BlockHeight> {
    let last_block = RPC
        .shell()
        .last_block(client)
        .await
        .context("Failed to query Namada's last committed block")?;

    Ok(last_block
        .map(|b| b.height.0 as BlockHeight)
        .unwrap_or_default())
}

// TODO: this can be improved / optimized(bonds and unbonds can be processed in
// parallel)
pub async fn query_all_bonds_and_unbonds(
    client: &HttpClient,
) -> anyhow::Result<(Bonds, Unbonds)> {
    type Source = NamadaSdkAddress;
    type Validator = NamadaSdkAddress;
    type WithdrawEpoch = NamadaSdkEpoch;

    type BondKey = (Source, Validator);
    type BondsMap = HashMap<BondKey, NamadaSdkAmount>;

    type UnbondKey = (Source, Validator, WithdrawEpoch);
    type UnbondsMap = HashMap<UnbondKey, NamadaSdkAmount>;

    let bonds_and_unbonds = bonds_and_unbonds(client, &None, &None)
        .await
        .context("Failed to query all bonds and unbonds")?;
    tracing::info!("bonds_and_unbonds {:?}", bonds_and_unbonds);

    let mut bonds: BondsMap = HashMap::new();
    let mut unbonds: UnbondsMap = HashMap::new();

    // This is not super nice but it's fewer iteratirons that doing map and then
    // reduce
    for (bond_id, details) in bonds_and_unbonds {
        for bd in details.bonds {
            let id = bond_id.clone();
            let key = (id.source, id.validator);

            if let Some(record) = bonds.get_mut(&key) {
                *record = record.checked_add(bd.amount).unwrap();
            } else {
                bonds.insert(key, bd.amount);
            }
        }

        for ud in details.unbonds {
            let id = bond_id.clone();
            let key = (id.source, id.validator, ud.withdraw);

            if let Some(record) = unbonds.get_mut(&key) {
                *record = record.checked_add(ud.amount).unwrap();
            } else {
                unbonds.insert(key, ud.amount);
            }
        }
    }

    // Map the types, mostly because we can't add indexer amounts
    let bonds = bonds
        .into_iter()
        .map(|((source, target), amount)| Bond {
            source: Id::from(source),
            target: Id::from(target),
            amount: Amount::from(amount),
        })
        .collect();

    let unbonds = unbonds
        .into_iter()
        .map(|((source, target, withdraw), amount)| Unbond {
            source: Id::from(source),
            target: Id::from(target),
            amount: Amount::from(amount),
            withdraw_at: withdraw.0 as Epoch,
        })
        .collect();

    Ok((bonds, unbonds))
}

pub async fn query_all_proposals(
    client: &HttpClient,
) -> anyhow::Result<Vec<GovernanceProposal>> {
    let last_proposal_id_key =
        namada_governance::storage::keys::get_counter_key();
    let last_proposal_id: u64 =
        query_storage_value(client, &last_proposal_id_key)
            .await
            .unwrap();

    let mut proposals: Vec<GovernanceProposal> = vec![];

    for id in 0..last_proposal_id {
        let proposal = query_proposal_by_id(client, id)
            .await
            .unwrap()
            .expect("Proposal should be written to storage.");
        let proposal_type = proposal.r#type.clone();

        // Create a governance proposal from the namada proposal, without the
        // data
        let mut governance_proposal = GovernanceProposal::from(proposal);

        // Get the proposal data based on the proposal type
        let proposal_data = match proposal_type {
            namada_governance::ProposalType::DefaultWithWasm(_) => {
                let wasm_code = query_proposal_code(client, id).await?;
                let hex_encoded = String::from_utf8(hex::encode(wasm_code))
                    .unwrap_or_default();
                Some(hex_encoded)
            }
            namada_governance::ProposalType::PGFSteward(data) => {
                Some(serde_json::to_string(&data).unwrap())
            }
            namada_governance::ProposalType::PGFPayment(data) => {
                Some(serde_json::to_string(&data).unwrap())
            }
            namada_governance::ProposalType::Default => None,
        };

        // Add the proposal data to the governance proposal
        governance_proposal.data = proposal_data;

        proposals.push(governance_proposal);
    }

    anyhow::Ok(proposals)
}

pub async fn query_proposal_code(
    client: &HttpClient,
    proposal_id: u64,
) -> anyhow::Result<Vec<u8>> {
    let proposal_code_key =
        namada_governance::storage::keys::get_proposal_code_key(proposal_id);
    let proposal_code = query_storage_value(client, &proposal_code_key)
        .await
        .expect("Proposal code should be written to storage.");

    anyhow::Ok(proposal_code)
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
    borsh::BorshDeserialize::try_from_slice(&query_result.data)
        .context("Failed to deserialize proposal id")
}

pub async fn query_bonds(
    client: &HttpClient,
    addresses: Vec<BondAddresses>,
    epoch: Epoch,
) -> anyhow::Result<Bonds> {
    let mut bonds = vec![];

    for BondAddresses { source, target } in addresses {
        let source = NamadaSdkAddress::from_str(&source.to_string())
            .expect("Failed to parse source address");
        let target = NamadaSdkAddress::from_str(&target.to_string())
            .expect("Failed to parse target address");

        let amount = RPC
            .vp()
            .pos()
            .bond_with_slashing(
                client,
                &source,
                &target,
                // TODO: + 2 is hardcoded pipeline len
                &Some(to_epoch(epoch + 2)),
            )
            .await
            .context("Failed to query bond amount")?;

        bonds.push(Bond {
            source: Id::from(source),
            target: Id::from(target),
            amount: Amount::from(amount),
        });
    }

    anyhow::Ok(bonds)
}

pub async fn query_unbonds(
    client: &HttpClient,
    addresses: Vec<UnbondAddresses>,
) -> anyhow::Result<Unbonds> {
    let mut unbonds = vec![];

    for UnbondAddresses { source, validator } in addresses {
        let source = NamadaSdkAddress::from_str(&source.to_string())
            .context("Failed to parse source address")?;
        let validator = NamadaSdkAddress::from_str(&validator.to_string())
            .context("Failed to parse validator address")?;

        let res = rpc::query_unbond_with_slashing(client, &source, &validator)
            .await
            .context("Failed to query unbond amount")?;

        let ((_, withdraw_epoch), amount) =
            res.last().context("Unbonds are empty")?;

        unbonds.push(Unbond {
            source: Id::from(source),
            target: Id::from(validator),
            amount: Amount::from(*amount),
            withdraw_at: withdraw_epoch.0 as Epoch,
        });
    }

    anyhow::Ok(unbonds)
}

pub async fn get_current_epoch(client: &HttpClient) -> anyhow::Result<Epoch> {
    let epoch = rpc::query_epoch(client)
        .await
        .context("Failed to query Namada's current epoch")?;

    Ok(epoch.0 as Epoch)
}

pub async fn query_tx_code_hash(
    client: &HttpClient,
    tx_code_path: &str,
) -> Option<String> {
    let hash_key = Key::wasm_hash(tx_code_path);
    let (tx_code_res, _) =
        rpc::query_storage_value_bytes(client, &hash_key, None, false)
            .await
            .ok()?;
    if let Some(tx_code_bytes) = tx_code_res {
        let tx_code =
            Hash::try_from(&tx_code_bytes[..]).expect("Invalid code hash");
        Some(tx_code.to_string())
    } else {
        None
    }
}

fn to_block_height(block_height: u32) -> NamadaSdkBlockHeight {
    NamadaSdkBlockHeight::from(block_height as u64)
}

fn to_epoch(epoch: u32) -> NamadaSdkEpoch {
    NamadaSdkEpoch::from(epoch as u64)
}
