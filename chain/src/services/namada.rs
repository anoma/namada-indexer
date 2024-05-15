use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use namada_core::storage::{
    BlockHeight as NamadaSdkBlockHeight, Epoch as NamadaSdkEpoch,
};
use namada_sdk::address::Address as NamadaSdkAddress;
use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::queries::RPC;
use namada_sdk::rpc::{
    bonds_and_unbonds, query_proposal_by_id, query_storage_value,
};
use namada_sdk::{rpc, token};
use shared::balance::{Amount, Balance, Balances};
use shared::block::{BlockHeight, Epoch};
use shared::bond::{Bond, BondAddresses, Bonds};
use shared::bond::{Bond, BondAddresses, Bonds};
use shared::id::Id;
use shared::proposal::GovernanceProposal;
use shared::unbond::{Unbond, UnbondAddresses, Unbonds};
use shared::utils::BalanceChange;
use shared::vote::{GovernanceVote, ProposalVoteKind};
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

// TODO: remove unwraps
pub async fn query_balance(
    client: &HttpClient,
    balance_changes: &HashSet<BalanceChange>,
) -> anyhow::Result<Balances> {
    let mut res: Balances = vec![];

    for balance_change in balance_changes {
        let owner =
            NamadaSdkAddress::from_str(&balance_change.address.to_string())
                .context("Failed to parse owner address")?;
        let token =
            NamadaSdkAddress::from_str(&balance_change.token.to_string())
                .context("Failed to parse token address")?;

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
            //TODO: unwrap
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

pub async fn query_all_bonds_and_unbonds(
    client: &HttpClient,
) -> anyhow::Result<(Bonds, Unbonds)> {
    let bonds_and_unbonds = bonds_and_unbonds(client, &None, &None)
        .await
        .context("Failed to query all bonds and unbonds")?;
    let mut bonds = vec![];
    let mut unbonds = vec![];

    for (id, details) in bonds_and_unbonds {
        for bond_details in details.bonds {
            bonds.push(Bond {
                epoch: bond_details.start.0 as Epoch,
                source: Id::from(id.source.clone()),
                target: Id::from(id.validator.clone()),
                amount: Amount::from(bond_details.amount),
            });
        }

        for unbond_details in details.unbonds {
            unbonds.push(Unbond {
                epoch: unbond_details.start.0 as Epoch,
                source: Id::from(id.source.clone()),
                target: Id::from(id.validator.clone()),
                amount: Amount::from(unbond_details.amount),
                withdraw_at: unbond_details.withdraw.0 as Epoch,
            });
        }
    }

    Ok((bonds, unbonds))
}

pub async fn query_all_votes(
    client: &HttpClient,
    proposal_ids: Vec<u64>,
) -> anyhow::Result<Vec<GovernanceVote>> {
    let mut res = vec![];
    for proposal_id in proposal_ids {
        let votes = namada_sdk::rpc::query_proposal_votes(client, proposal_id)
            .await
            .unwrap();

        // TODO: maybe just use for
        let votes = votes
            .iter()
            .cloned()
            .map(|v| GovernanceVote {
                proposal_id,
                vote: ProposalVoteKind::from(v.data),
                address: Id::from(v.delegator),
            })
            .collect::<Vec<GovernanceVote>>();

        res.push(votes);
    }

    // TODO: not sure if this is optimal
    let res = res.into_iter().flatten().collect();

    anyhow::Ok(res)
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

        // Create a governance proposal from the namada proposal, without the data
        let mut governance_proposal = GovernanceProposal::from(proposal);

        // Get the proposal data based on the proposal type
        let proposal_data = match proposal_type {
            namada_governance::ProposalType::DefaultWithWasm(_) => {
                Some(query_proposal_code(client, id).await?)
            }
            namada_governance::ProposalType::PGFSteward(data) => {
                Some(data.serialize_to_vec()) // maybe change to json or some other encoding ?
            }
            namada_governance::ProposalType::PGFPayment(data) => {
                Some(data.serialize_to_vec()) // maybe change to json or some other encoding ?
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
            epoch,
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
    epoch: Epoch,
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

        tracing::info!("unbonds {:?}", res);

        let ((_, withdraw_epoch), amount) =
            res.last().context("Unbonds are empty")?;

        unbonds.push(Unbond {
            epoch,
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

fn to_block_height(block_height: u32) -> NamadaSdkBlockHeight {
    NamadaSdkBlockHeight::from(block_height as u64)
}

fn to_epoch(epoch: u32) -> NamadaSdkEpoch {
    NamadaSdkEpoch::from(epoch as u64)
}
