use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

use namada_sdk::borsh::BorshDeserialize;
use subtle_encoding::hex;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;

use crate::block_result::BlockResult;
use crate::bond::BondAddresses;
use crate::checksums::Checksums;
use crate::header::BlockHeader;
use crate::id::Id;
use crate::proposal::{GovernanceProposal, GovernanceProposalKind};
use crate::public_key::PublicKey;
use crate::transaction::{
    InnerTransaction, Transaction, TransactionKind, WrapperTransaction,
};
use crate::unbond::UnbondAddresses;
use crate::utils::BalanceChange;
use crate::validator::ValidatorMetadataChange;
use crate::vote::GovernanceVote;

pub type Epoch = u32;
pub type BlockHeight = u32;

#[derive(Debug, Clone)]
pub enum EventKind {
    Applied,
    Rejected,
    Accepted,
    Unknown,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TxEventStatusCode {
    Ok,
    #[default]
    Fail,
}

impl From<&str> for TxEventStatusCode {
    fn from(value: &str) -> Self {
        match value {
            "0" => Self::Ok,
            _ => Self::Fail,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TxAttributes {
    pub code: TxEventStatusCode,
    pub gas: u64,
    pub hash: Id,
    pub height: u64,
    pub info: String,
}

impl TxAttributes {
    pub fn deserialize(
        event_kind: &EventKind,
        attributes: &BTreeMap<String, String>,
    ) -> Self {
        match event_kind {
            EventKind::Unknown => Self::default(),
            _ => Self {
                code: attributes
                    .get("code")
                    .map(|code| TxEventStatusCode::from(code.as_str()))
                    .unwrap()
                    .to_owned(),
                gas: attributes
                    .get("gas_used")
                    .map(|gas| u64::from_str(gas).unwrap())
                    .unwrap()
                    .to_owned(),
                hash: attributes
                    .get("hash")
                    .map(|hash| Id::Hash(hash.to_lowercase()))
                    .unwrap()
                    .to_owned(),
                height: attributes
                    .get("height")
                    .map(|height| u64::from_str(height).unwrap())
                    .unwrap()
                    .to_owned(),
                info: attributes.get("info").unwrap().to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub transactions: Vec<(WrapperTransaction, Vec<InnerTransaction>)>,
    pub epoch: Epoch,
}

impl Block {
    pub fn from(
        block_response: TendermintBlockResponse,
        block_results: &BlockResult,
        checksums: Checksums,
        epoch: Epoch,
        block_height: BlockHeight,
    ) -> Self {
        let transactions = block_response
            .block
            .data
            .iter()
            .enumerate()
            .filter_map(|(index, tx_raw_bytes)| {
                Transaction::deserialize(
                    tx_raw_bytes,
                    index,
                    block_height,
                    checksums.clone(),
                    block_results,
                )
                .map_err(|reason| {
                    tracing::info!("Couldn't deserialize tx due to {}", reason);
                })
                .ok()
            })
            .collect::<Vec<(WrapperTransaction, Vec<InnerTransaction>)>>();

        Block {
            hash: Id::from(block_response.block_id.hash),
            header: BlockHeader {
                height: block_response.block.header.height.value()
                    as BlockHeight,
                proposer_address: block_response
                    .block
                    .header
                    .proposer_address
                    .to_string()
                    .to_lowercase(),
                timestamp: block_response.block.header.time.to_string(),
                app_hash: Id::from(block_response.block.header.app_hash),
            },
            transactions,
            epoch,
        }
    }

    pub fn inner_txs(&self) -> Vec<InnerTransaction> {
        self.transactions
            .iter()
            .flat_map(|(_, inner_txs)| inner_txs.clone())
            .collect()
    }

    pub fn wrapper_txs(&self) -> Vec<WrapperTransaction> {
        self.transactions
            .iter()
            .map(|(wrapper_tx, _)| wrapper_tx.clone())
            .collect()
    }

    pub fn governance_proposal(
        &self,
        mut next_proposal_id: u64,
    ) -> Vec<GovernanceProposal> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::InitProposal(data) => {
                    let init_proposal_data = data.clone();

                    let proposal_content_bytes = tx
                        .get_section_data_by_id(Id::from(
                            init_proposal_data.content,
                        ))
                        .unwrap_or_default();

                    let proposal_content =
                        BTreeMap::<String, String>::try_from_slice(
                            &proposal_content_bytes,
                        )
                        .unwrap_or_default();

                    let proposal_content_serialized =
                        serde_json::to_string_pretty(&proposal_content)
                            .unwrap_or_default();

                    let proposal_data = match init_proposal_data.r#type.clone()
                    {
                        namada_governance::ProposalType::DefaultWithWasm(
                            hash,
                        ) => {
                            let wasm_code =
                                tx.get_section_data_by_id(Id::from(hash));
                            if let Some(wasm_code) = wasm_code {
                                let hex_encoded =
                                    String::from_utf8(hex::encode(wasm_code))
                                        .unwrap_or_default();
                                Some(hex_encoded)
                            } else {
                                None
                            }
                        }
                        namada_governance::ProposalType::PGFSteward(data) => {
                            Some(serde_json::to_string(&data).unwrap())
                        }
                        namada_governance::ProposalType::PGFPayment(data) => {
                            Some(serde_json::to_string(&data).unwrap())
                        }
                        namada_governance::ProposalType::Default => None,
                    };

                    let current_id = next_proposal_id;
                    next_proposal_id += 1;

                    Some(GovernanceProposal {
                        id: current_id,
                        author: Id::from(init_proposal_data.author),
                        r#type: GovernanceProposalKind::from(
                            init_proposal_data.r#type,
                        ),
                        data: proposal_data,
                        voting_start_epoch: Epoch::from(
                            init_proposal_data.voting_start_epoch.0 as u32,
                        ),
                        voting_end_epoch: Epoch::from(
                            init_proposal_data.voting_end_epoch.0 as u32,
                        ),
                        activation_epoch: Epoch::from(
                            init_proposal_data.activation_epoch.0 as u32,
                        ),
                        content: proposal_content_serialized,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn pos_rewards(&self) -> HashSet<Id> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::ClaimRewards(data) => {
                    let data = data.clone();
                    let source = data.source.unwrap_or(data.validator);

                    Some(Id::from(source))
                }
                _ => None,
            })
            .collect()
    }

    pub fn governance_votes(&self) -> Vec<GovernanceVote> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::ProposalVote(data) => {
                    let vote_proposal_data = data.clone();

                    Some(GovernanceVote {
                        proposal_id: vote_proposal_data.id,
                        vote: vote_proposal_data.vote.into(),
                        address: Id::from(vote_proposal_data.voter),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn addresses_with_balance_change(
        &self,
        native_token: Id,
    ) -> HashSet<BalanceChange> {
        self.transactions
            .iter()
            .flat_map(|(wrapper_tx, inners_txs)| {
                let mut balance_changes = vec![];
                for tx in inners_txs {
                    let mut balance_change = match &tx.kind {
                        TransactionKind::TransparentTransfer(data) => {
                            let transfer_data = data.clone();
                            let mut changes = vec![];
    
                            // Iterate over sources in the BTreeMap
                            for (source, _amount) in transfer_data.sources.iter() {
                                let transfer_source = Id::from(source.owner.clone());
                                changes.push(BalanceChange::new(
                                    transfer_source,
                                    native_token.clone(),
                                ));
                            }
    
                            // Iterate over targets in the BTreeMap
                            for (target, _amount) in transfer_data.targets.iter() {
                                let transfer_target = Id::from(target.owner.clone());
                                changes.push(BalanceChange::new(
                                    transfer_target,
                                    native_token.clone(),
                                ));
                            }
    
                            changes
                        }
                        TransactionKind::Bond(data) => {
                            let bond_data = data.clone();
                            let address = bond_data.source.unwrap_or(bond_data.validator);
                            let source = Id::from(address);
                            vec![BalanceChange::new(
                                source,
                                native_token.clone(),
                            )]
                        }
                        TransactionKind::Withdraw(data) => {
                            let withdraw_data = data.clone();
                            let address = withdraw_data
                                .source
                                .unwrap_or(withdraw_data.validator);
                            let source = Id::from(address);
    
                            vec![BalanceChange::new(
                                source,
                                native_token.clone(),
                            )]
                        }
                        TransactionKind::ClaimRewards(data) => {
                            let claim_rewards_data = data.clone();
                            let address = claim_rewards_data
                                .source
                                .unwrap_or(claim_rewards_data.validator);
                            let source = Id::from(address);
    
                            vec![BalanceChange::new(
                                source,
                                native_token.clone(),
                            )]
                        }
                        TransactionKind::InitProposal(data) => {
                            let init_proposal_data = data.clone();
                            let author = Id::from(init_proposal_data.author);
    
                            vec![BalanceChange::new(
                                author,
                                native_token.clone(),
                            )]
                        }
                        _ => vec![],
                    };
                    balance_changes.append(&mut balance_change);
                }
    
                balance_changes.push(BalanceChange::new(
                    wrapper_tx.fee.gas_payer.clone(),
                    wrapper_tx.fee.gas_token.clone(),
                ));
                balance_changes
            })
            .collect()
    }  

    pub fn bond_addresses(&self) -> Vec<BondAddresses> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::Bond(data) => {
                    let bond_data = data.clone();
                    let source_address =
                        bond_data.source.unwrap_or(bond_data.validator.clone());
                    let target_address = bond_data.validator;

                    Some(vec![BondAddresses {
                        source: Id::from(source_address),
                        target: Id::from(target_address),
                    }])
                }
                TransactionKind::Unbond(data) => {
                    let unbond_data = data.clone();
                    let source_address = unbond_data
                        .source
                        .unwrap_or(unbond_data.validator.clone());
                    let validator_address = unbond_data.validator;

                    Some(vec![BondAddresses {
                        source: Id::from(source_address),
                        target: Id::from(validator_address),
                    }])
                }
                TransactionKind::Redelegation(data) => {
                    let redelegation_data = data.clone();
                    let owner = redelegation_data.owner;
                    let source_validator = redelegation_data.src_validator;
                    let destination_validator =
                        redelegation_data.dest_validator;

                    Some(vec![
                        BondAddresses {
                            source: Id::from(owner.clone()),
                            target: Id::from(source_validator),
                        },
                        BondAddresses {
                            source: Id::from(owner),
                            target: Id::from(destination_validator),
                        },
                    ])
                }
                _ => None,
            })
            .flatten()
            .collect()
    }

    pub fn unbond_addresses(&self) -> Vec<UnbondAddresses> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::Unbond(data) => {
                    let unbond_data = data.clone();

                    let source_address = unbond_data
                        .source
                        .unwrap_or(unbond_data.validator.clone());
                    let validator_address = unbond_data.validator;

                    Some(UnbondAddresses {
                        source: Id::from(source_address),
                        validator: Id::from(validator_address),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn withdraw_addresses(&self) -> Vec<UnbondAddresses> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::Withdraw(data) => {
                    let withdraw_data = data.clone();

                    let source_address = withdraw_data
                        .source
                        .unwrap_or(withdraw_data.validator.clone());
                    let validator_address = withdraw_data.validator;

                    Some(UnbondAddresses {
                        source: Id::from(source_address),
                        validator: Id::from(validator_address),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn validator_metadata(&self) -> Vec<ValidatorMetadataChange> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::MetadataChange(data) => {
                    let metadata_change_data = data.clone();

                    let source_address = metadata_change_data.validator;

                    Some(ValidatorMetadataChange {
                        address: Id::from(source_address),
                        commission: metadata_change_data
                            .commission_rate
                            .map(|c| c.to_string()),
                        name: metadata_change_data.name,
                        email: metadata_change_data.email,
                        description: metadata_change_data.description,
                        website: metadata_change_data.website,
                        discord_handler: metadata_change_data.discord_handle,
                        avatar: metadata_change_data.avatar,
                    })
                }
                TransactionKind::CommissionChange(data) => {
                    let commission_change = data.clone();

                    let source_address = commission_change.validator;

                    Some(ValidatorMetadataChange {
                        address: Id::from(source_address),
                        commission: Some(
                            commission_change.new_rate.to_string(),
                        ),
                        name: None,
                        email: None,
                        description: None,
                        website: None,
                        discord_handler: None,
                        avatar: None,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn revealed_pks(&self) -> Vec<(PublicKey, Id)> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter_map(|tx| match &tx.kind {
                TransactionKind::RevealPk(data) => {
                    let namada_public_key = data.clone();

                    Some((
                        PublicKey::from(namada_public_key.clone()),
                        Id::from(namada_public_key),
                    ))
                }
                _ => None,
            })
            .collect()
    }
}
