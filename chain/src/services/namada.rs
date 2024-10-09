use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use futures::{StreamExt, TryStreamExt};
use namada_core::chain::{
    BlockHeight as NamadaSdkBlockHeight, Epoch as NamadaSdkEpoch,
};
use namada_sdk::address::{Address as NamadaSdkAddress, InternalAddress};
use namada_sdk::collections::HashMap;
use namada_sdk::hash::Hash;
use namada_sdk::ibc::storage::{ibc_trace_key_prefix, is_ibc_trace_key};
use namada_sdk::ibc::IbcTokenHash;
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
use shared::proposal::{GovernanceProposal, TallyType};
use shared::token::{IbcToken, Token};
use shared::unbond::{Unbond, UnbondAddresses, Unbonds};
use shared::utils::BalanceChange;
use shared::validator::{Validator, ValidatorSet, ValidatorState};
use shared::vote::{GovernanceVote, ProposalVoteKind};
use subtle_encoding::hex;
use tendermint_rpc::HttpClient;

use super::utils::query_storage_prefix;

pub async fn get_native_token(client: &HttpClient) -> anyhow::Result<Id> {
    let native_token = RPC
        .shell()
        .native_token(client)
        .await
        .context("Failed to query native token")?;
    Ok(Id::from(native_token))
}

pub async fn get_first_block_in_epoch(
    client: &HttpClient,
) -> anyhow::Result<BlockHeight> {
    let block_height = RPC
        .shell()
        .first_block_height_of_current_epoch(client)
        .await
        .context("Failed to query native token")?;

    Ok(block_height.0 as BlockHeight)
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

            let token_addr = match &balance_change.token {
                Token::Ibc(IbcToken { address, .. }) => address.clone(),
                Token::Native(addr) => addr.clone(),
            }
            .into();

            let amount =
                rpc::get_token_balance(client, &token_addr, &owner, None)
                    .await
                    .unwrap_or_default();

            Some(Balance {
                owner: balance_change.address.clone(),
                token: balance_change.token.clone(),
                amount: Amount::from(amount),
            })
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await)
}

pub async fn query_tokens(client: &HttpClient) -> anyhow::Result<Vec<Token>> {
    let ibc_tokens = query_ibc_tokens(client).await?;
    let native_token = RPC
        .shell()
        .native_token(client)
        .await
        .context("Failed to query native token")?;

    let tokens = ibc_tokens
        .into_iter()
        .map(|ibc_token| {
            Token::Ibc(IbcToken {
                address: ibc_token.address,
                trace: ibc_token.trace,
            })
        })
        .chain(std::iter::once(Token::Native(Id::from(native_token))))
        .collect::<Vec<_>>();

    Ok(tokens)
}

async fn query_ibc_tokens(
    client: &HttpClient,
) -> anyhow::Result<HashSet<IbcToken>> {
    let prefix = ibc_trace_key_prefix(None);

    let mut tokens: HashSet<IbcToken> = HashSet::new();
    let ibc_traces = query_storage_prefix::<String>(client, &prefix).await?;
    if let Some(ibc_traces) = ibc_traces {
        for (key, ibc_trace) in ibc_traces {
            if let Some((_, hash)) = is_ibc_trace_key(&key) {
                let hash: IbcTokenHash = hash.parse().expect(
                    "Parsing an IBC token hash from storage shouldn't fail",
                );
                let ibc_token_addr =
                    NamadaSdkAddress::Internal(InternalAddress::IbcToken(hash));

                let token = IbcToken {
                    address: Id::from(ibc_token_addr),
                    trace: Id::IbcTrace(ibc_trace.clone()),
                };

                tokens.insert(token);
            }
        }
    }

    Ok(tokens)
}

pub async fn query_all_balances(
    client: &HttpClient,
) -> anyhow::Result<Balances> {
    let tokens = query_tokens(client).await?;
    let mut all_balances: Balances = vec![];

    for token in tokens.into_iter() {
        let balances = add_balance(client, token).await?;
        all_balances.extend(balances);
    }

    anyhow::Ok(all_balances)
}

async fn add_balance(
    client: &HttpClient,
    token: Token,
) -> anyhow::Result<Vec<Balance>> {
    let mut all_balances: Vec<Balance> = vec![];
    let token_addr = match token {
        Token::Ibc(IbcToken { ref address, .. }) => address.clone(),
        Token::Native(ref addr) => addr.clone(),
    };

    let balance_prefix = namada_token::storage_key::balance_prefix(
        &NamadaSdkAddress::from(token_addr),
    );

    let balances =
        query_storage_prefix::<token::Amount>(client, &balance_prefix)
            .await
            .context("Failed to query all balances")?;

    if let Some(balances) = balances {
        for (key, balance) in balances {
            let (o, b) =
                match namada_token::storage_key::is_any_token_balance_key(&key)
                {
                    Some([_, owner]) => (owner.clone(), balance),
                    None => continue,
                };

            all_balances.push(Balance {
                owner: Id::from(o),
                token: token.clone(),
                amount: Amount::from(b),
            })
        }
    }

    Ok(all_balances)
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
    source: Option<Id>,
    target: Option<Id>,
) -> anyhow::Result<(Bonds, Unbonds)> {
    type Source = NamadaSdkAddress;
    type Validator = NamadaSdkAddress;
    type WithdrawEpoch = NamadaSdkEpoch;
    type StartEpoch = NamadaSdkEpoch;

    type BondKey = (Source, Validator, StartEpoch);
    type BondsMap = HashMap<BondKey, NamadaSdkAmount>;

    type UnbondKey = (Source, Validator, WithdrawEpoch);
    type UnbondsMap = HashMap<UnbondKey, NamadaSdkAmount>;

    let bonds_and_unbonds = bonds_and_unbonds(
        client,
        &source.map(NamadaSdkAddress::from),
        &target.map(NamadaSdkAddress::from),
    )
    .await
    .context("Failed to query all bonds and unbonds")?;

    let mut bonds: BondsMap = HashMap::new();
    let mut unbonds: UnbondsMap = HashMap::new();

    // This is not super nice but it's fewer iterations that doing map and then
    // reduce
    for (bond_id, details) in bonds_and_unbonds {
        for bd in details.bonds {
            let id = bond_id.clone();
            let key = (id.source, id.validator, bd.start);

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
        .map(|((source, target, start), amount)| Bond {
            source: Id::from(source),
            target: Id::from(target),
            amount: Amount::from(amount),
            start: start.0 as Epoch,
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
    // For block_height 0 the next id is always 0
    if block_height == 0 {
        return Ok(0);
    }
    // For all other block heights we need to subtract 1
    // as namada already saved current block and bumped next proposal id
    let block_height = block_height - 1;

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
    addresses: HashSet<BondAddresses>,
) -> anyhow::Result<Vec<(Id, Id, Option<Bond>)>> {
    let nested_bonds = futures::stream::iter(addresses)
        .filter_map(|BondAddresses { source, target }| async move {
            // TODO: if this is too slow do not use query_all_bonds_and_unbonds
            let (bonds_res, _) = query_all_bonds_and_unbonds(
                client,
                Some(source.clone()),
                Some(target.clone()),
            )
            .await
            .context("Failed to query all bonds and unbonds")
            .ok()?;

            let bonds = if !bonds_res.is_empty() {
                bonds_res
                    .into_iter()
                    .map(|bond| (source.clone(), target.clone(), Some(bond)))
                    .collect::<Vec<_>>()
            } else {
                vec![(source, target, None)]
            };

            Some(bonds)
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await;

    let bonds = nested_bonds.iter().flatten().cloned().collect();

    anyhow::Ok(bonds)
}

pub async fn query_unbonds(
    client: &HttpClient,
    addresses: HashSet<UnbondAddresses>,
) -> anyhow::Result<Unbonds> {
    let nested_unbonds = futures::stream::iter(addresses)
        .filter_map(|UnbondAddresses { source, validator }| {
            let source = NamadaSdkAddress::from_str(&source.to_string())
                .expect("Failed to parse source address");
            let validator = NamadaSdkAddress::from_str(&validator.to_string())
                .expect("Failed to parse validator address");

            async move {
                let unbonds = RPC
                    .vp()
                    .pos()
                    .unbond_with_slashing(client, &source, &validator)
                    .await
                    .context("Failed to query unbond amount")
                    .ok()?;

                let mut unbonds_map: HashMap<(Id, Id, Epoch), Amount> =
                    HashMap::new();

                for ((_, withdraw_epoch), amount) in unbonds {
                    let record = unbonds_map.get_mut(&(
                        Id::from(source.clone()),
                        Id::from(validator.clone()),
                        withdraw_epoch.0 as Epoch,
                    ));

                    // We have  to merge the unbonds with the same withdraw
                    // epoch into one otherwise we can't
                    // insert them into the db
                    match record {
                        Some(r) => {
                            *r = r.checked_add(&Amount::from(amount)).unwrap();
                        }
                        None => {
                            unbonds_map.insert(
                                (
                                    Id::from(source.clone()),
                                    Id::from(validator.clone()),
                                    withdraw_epoch.0 as Epoch,
                                ),
                                Amount::from(amount),
                            );
                        }
                    }
                }

                let unbonds: Vec<Unbond> = unbonds_map
                    .into_iter()
                    .map(|((source, target, start), amount)| Unbond {
                        source,
                        target,
                        amount,
                        withdraw_at: start,
                    })
                    .collect();

                Some(unbonds)
            }
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await;

    let unbonds = nested_unbonds.iter().flatten().cloned().collect();

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

pub async fn is_steward(
    client: &HttpClient,
    address: &Id,
) -> anyhow::Result<bool> {
    let address = NamadaSdkAddress::from_str(&address.to_string())
        .context("Failed to parse address")?;

    let is_steward = rpc::is_steward(client, &address).await;

    Ok(is_steward)
}

pub async fn query_tallies(
    client: &HttpClient,
    proposals: Vec<GovernanceProposal>,
) -> anyhow::Result<Vec<(GovernanceProposal, TallyType)>> {
    let proposals = futures::stream::iter(proposals)
        .filter_map(|proposal| async move {
            let is_steward = is_steward(client, &proposal.author).await.ok()?;

            let tally_type = TallyType::from(&proposal.r#type, is_steward);

            Some((proposal, tally_type))
        })
        .map(futures::future::ready)
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await;

    anyhow::Ok(proposals)
}

pub async fn query_all_votes(
    client: &HttpClient,
    proposals_ids: Vec<u64>,
) -> anyhow::Result<HashSet<GovernanceVote>> {
    let votes: Vec<HashSet<GovernanceVote>> =
        futures::stream::iter(proposals_ids)
            .filter_map(|proposal_id| async move {
                let votes = rpc::query_proposal_votes(client, proposal_id)
                    .await
                    .ok()?;

                let votes = votes
                    .into_iter()
                    .map(|vote| GovernanceVote {
                        proposal_id,
                        vote: ProposalVoteKind::from(vote.data),
                        address: Id::from(vote.delegator),
                    })
                    .collect::<HashSet<_>>();

                Some(votes)
            })
            .map(futures::future::ready)
            .buffer_unordered(20)
            .collect::<Vec<_>>()
            .await;

    anyhow::Ok(votes.iter().flatten().cloned().collect())
}

pub async fn get_validator_set_at_epoch(
    client: &HttpClient,
    epoch: Epoch,
) -> anyhow::Result<ValidatorSet> {
    let namada_epoch = NamadaSdkEpoch::from(epoch as u64);
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

            let commission_fut = async {
                rpc::query_commission_rate(client, &address, Some(namada_epoch))
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to query commission of validator \
                             {address} at epoch {namada_epoch}"
                        )
                    })
            };

            let validator_state_fut = async {
                rpc::get_validator_state(client, &address, Some(namada_epoch))
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to query validator {address} \
                         state"
                        )
                    })
            };

            let (voting_power, commission_pair, validator_state) =
                futures::try_join!(voting_power_fut, commission_fut, validator_state_fut)?;
            let commission = commission_pair
                .commission_rate
                .expect("Commission rate has to exist")
                .to_string();
            let max_commission = commission_pair
                .max_commission_change_per_epoch
                .expect("Max commission rate change has to exist")
                .to_string();
            let validator_state = validator_state.0.map(ValidatorState::from).unwrap_or(ValidatorState::Unknown);

            anyhow::Ok(Validator {
                address: Id::Account(address.to_string()),
                voting_power: voting_power.to_string_native(),
                max_commission,
                commission,
                name: None,
                email: None,
                description: None,
                website: None,
                discord_handler: None,
                avatar: None,
                state: validator_state
            })
        })
        .buffer_unordered(100)
        .try_collect::<HashSet<_>>()
        .await?;

    Ok(ValidatorSet { validators, epoch })
}

pub async fn query_pipeline_length(client: &HttpClient) -> anyhow::Result<u64> {
    let pos_parameters = rpc::get_pos_params(client)
        .await
        .with_context(|| "Failed to query pos parameters".to_string())?;

    Ok(pos_parameters.pipeline_len)
}

fn to_block_height(block_height: u32) -> NamadaSdkBlockHeight {
    NamadaSdkBlockHeight::from(block_height as u64)
}
