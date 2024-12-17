use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

use namada_ibc::apps::transfer::types::packet::PacketData;
use namada_ibc::core::channel::types::msgs::{MsgRecvPacket, PacketMsg};
use namada_ibc::core::handler::types::msgs::MsgEnvelope;
use namada_ibc::IbcMessage;
use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::token::Transfer;
use subtle_encoding::hex;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;

use crate::block_result::BlockResult;
use crate::bond::BondAddresses;
use crate::checksums::Checksums;
use crate::header::BlockHeader;
use crate::id::Id;
use crate::proposal::{GovernanceProposal, GovernanceProposalKind};
use crate::public_key::PublicKey;
use crate::token::{IbcToken, Token};
use crate::transaction::{
    InnerTransaction, Transaction, TransactionExitStatus, TransactionKind,
    WrapperTransaction,
};
use crate::unbond::UnbondAddresses;
use crate::utils::BalanceChange;
use crate::validator::{
    Validator, ValidatorMetadataChange, ValidatorState, ValidatorStateChange,
};
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
        block_response: &TendermintBlockResponse,
        block_results: &BlockResult,
        proposer_address_namada: &Option<Id>, /* Provide the namada address
                                               * of the proposer, if
                                               * available */
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
                proposer_address_tm: block_response
                    .block
                    .header
                    .proposer_address
                    .to_string()
                    .to_lowercase(),
                proposer_address_namada: proposer_address_namada
                    .as_ref()
                    .map(Id::to_string),
                timestamp: block_response.block.header.time.to_string(),
                app_hash: Id::from(&block_response.block.header.app_hash),
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
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::InitProposal(data) => {
                    let init_proposal_data = data.clone().unwrap(); // safe as we filter before (not the best pattern)

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

    pub fn pos_rewards(&self) -> HashSet<(Id, Id)> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::ClaimRewards(data) => {
                    let data = data.clone().unwrap();
                    let validator = data.validator;
                    let source = data.source.unwrap_or(validator.clone());

                    Some((Id::from(source), Id::from(validator)))
                }
                _ => None,
            })
            .collect()
    }

    pub fn governance_votes(&self) -> HashSet<GovernanceVote> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::ProposalVote(data) => {
                    let vote_proposal_data = data.clone().unwrap();

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

    pub fn ibc_tokens(&self) -> HashSet<IbcToken> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::IbcMsgTransfer(data) => {
                    let data = data.clone().and_then(|d| {
                        Self::ibc_msg_recv_packet(d.0).and_then(|msg| {
                            serde_json::from_slice::<PacketData>(
                                &msg.packet.data,
                            )
                            .map(|p| (msg, p))
                            .ok()
                        })
                    });

                    let (msg, packet_data) = data?;

                    let ibc_trace = format!(
                        "{}/{}/{}",
                        msg.packet.port_id_on_b,
                        msg.packet.chan_id_on_b,
                        packet_data.token.denom
                    );

                    let trace = Id::IbcTrace(ibc_trace.clone());
                    let address =
                        namada_ibc::trace::convert_to_address(ibc_trace)
                            .expect("Failed to convert IBC trace to address");
                    Some(IbcToken {
                        address: Id::from(address.clone()),
                        trace,
                    })
                }
                _ => None,
            })
            .collect()
    }

    // TODO: move this and process_inner_tx_for_balance to a separate module
    pub fn addresses_with_balance_change(
        &self,
        native_token: &Id,
    ) -> HashSet<BalanceChange> {
        self.transactions
            .iter()
            .flat_map(|(wrapper_tx, inners_txs)| {
                let mut balance_changes: Vec<BalanceChange> = inners_txs
                    .iter()
                    .filter_map(|tx| {
                        self.process_inner_tx_for_balance(tx, native_token)
                    })
                    .flatten()
                    .collect();

                balance_changes.push(BalanceChange::new(
                    wrapper_tx.fee.gas_payer.clone(),
                    Token::Native(wrapper_tx.fee.gas_token.clone()),
                ));

                balance_changes
            })
            .collect()
    }

    pub fn process_inner_tx_for_balance(
        &self,
        tx: &InnerTransaction,
        native_token: &Id,
    ) -> Option<Vec<BalanceChange>> {
        let change = match &tx.kind {
            TransactionKind::IbcMsgTransfer(data) => {
                let data = data.clone().and_then(|d| {
                    Self::ibc_msg_recv_packet(d.0).and_then(|msg| {
                        serde_json::from_slice::<PacketData>(&msg.packet.data)
                            .map(|p| (msg, p))
                            .ok()
                    })
                });

                let (msg, packet_data) = data?;
                let denom = packet_data.token.denom.to_string();

                // If the denom is the native token, we can just return the
                // receiver
                if denom.contains(&native_token.to_string()) {
                    vec![BalanceChange::new(
                        Id::Account(String::from(
                            packet_data.receiver.as_ref(),
                        )),
                        Token::Native(native_token.clone()),
                    )]
                } else {
                    let ibc_trace = format!(
                        "{}/{}/{}",
                        msg.packet.port_id_on_b,
                        msg.packet.chan_id_on_b,
                        packet_data.token.denom
                    );

                    let trace = Id::IbcTrace(ibc_trace.clone());
                    let address =
                        namada_ibc::trace::convert_to_address(ibc_trace)
                            .expect("Failed to convert IBC trace to address");

                    vec![BalanceChange::new(
                        Id::Account(String::from(
                            packet_data.receiver.as_ref(),
                        )),
                        Token::Ibc(IbcToken {
                            address: Id::from(address.clone()),
                            trace,
                        }),
                    )]
                }
            }
            TransactionKind::TransparentTransfer(data) => {
                let data = data.as_ref()?;

                [&data.sources, &data.targets]
                    .iter()
                    .flat_map(|transfer_changes| {
                        transfer_changes.0.keys().map(|account| {
                            BalanceChange::new(
                                Id::from(account.owner.clone()),
                                Token::Native(Id::from(account.token.clone())),
                            )
                        })
                    })
                    .collect()
            }
            TransactionKind::Bond(data) => {
                let data = data.as_ref()?;

                let bond_data = data.clone();
                let address = bond_data.source.unwrap_or(bond_data.validator);
                let source = Id::from(address);

                vec![BalanceChange::new(
                    source,
                    Token::Native(native_token.clone()),
                )]
            }
            TransactionKind::Withdraw(data) => {
                let data = data.as_ref()?;

                let withdraw_data = data.clone();
                let address =
                    withdraw_data.source.unwrap_or(withdraw_data.validator);
                let source = Id::from(address);

                vec![BalanceChange::new(
                    source,
                    Token::Native(native_token.clone()),
                )]
            }
            TransactionKind::ClaimRewards(data) => {
                let data = data.as_ref()?;

                let claim_rewards_data = data.clone();
                let address = claim_rewards_data
                    .source
                    .unwrap_or(claim_rewards_data.validator);
                let source = Id::from(address);

                vec![BalanceChange::new(
                    source,
                    Token::Native(native_token.clone()),
                )]
            }
            TransactionKind::InitProposal(data) => {
                let data = data.as_ref()?;

                let init_proposal_data = data.clone();
                let author = Id::from(init_proposal_data.author);

                vec![BalanceChange::new(
                    author,
                    Token::Native(native_token.clone()),
                )]
            }
            _ => vec![],
        };

        Some(change)
    }

    pub fn ibc_msg_recv_packet(
        // TODO: not sure if token::Transfer is the right type here
        msg: IbcMessage<Transfer>,
    ) -> Option<MsgRecvPacket> {
        // Early return if the message is not an Envelope
        let IbcMessage::Envelope(e) = msg else {
            return None;
        };

        // Early return if the envelope is not a Packet
        let MsgEnvelope::Packet(packet_msg) = *e else {
            return None;
        };

        // Early return if it's not a Recv message
        let PacketMsg::Recv(recv_msg) = packet_msg else {
            return None;
        };

        Some(recv_msg)
    }

    pub fn new_validators(&self) -> HashSet<Validator> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::BecomeValidator(data) => {
                    let data = data.clone().unwrap();
                    Some(Validator {
                        address: Id::from(data.address),
                        voting_power: "0".to_string(),
                        max_commission: data
                            .max_commission_rate_change
                            .to_string(),
                        commission: data.commission_rate.to_string(),
                        name: data.name,
                        email: Some(data.email),
                        description: data.description,
                        website: data.website,
                        discord_handler: data.discord_handle,
                        avatar: data.avatar,
                        state: ValidatorState::Inactive,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn update_validators_state(&self) -> HashSet<ValidatorStateChange> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::DeactivateValidator(data) => {
                    let data = data.clone().unwrap();
                    Some(ValidatorStateChange {
                        address: Id::from(data),
                        state: ValidatorState::Deactivating,
                    })
                }
                TransactionKind::ReactivateValidator(data) => {
                    let data = data.clone().unwrap();
                    Some(ValidatorStateChange {
                        address: Id::from(data),
                        state: ValidatorState::Reactivating,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn bond_addresses(&self) -> HashSet<BondAddresses> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::Bond(data) => {
                    let bond_data = data.clone().unwrap();
                    let source_address =
                        bond_data.source.unwrap_or(bond_data.validator.clone());
                    let target_address = bond_data.validator;

                    Some(vec![BondAddresses {
                        source: Id::from(source_address),
                        target: Id::from(target_address),
                    }])
                }
                TransactionKind::Unbond(data) => {
                    let unbond_data = data.clone().unwrap();
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
                    let redelegation_data = data.clone().unwrap();
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

    pub fn unbond_addresses(&self) -> HashSet<UnbondAddresses> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::Unbond(data) => {
                    let unbond_data = data.clone().unwrap();

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

    pub fn withdraw_addresses(&self) -> HashSet<UnbondAddresses> {
        self.transactions
            .iter()
            .flat_map(|(_, txs)| txs)
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::Withdraw(data) => {
                    let withdraw_data = data.clone().unwrap();

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
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::MetadataChange(data) => {
                    let metadata_change_data = data.clone().unwrap();

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
                    let commission_change = data.clone().unwrap();

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
            .filter(|tx| {
                tx.data.is_some()
                    && tx.exit_code == TransactionExitStatus::Applied
            })
            .filter_map(|tx| match &tx.kind {
                TransactionKind::RevealPk(data) => {
                    let namada_public_key = data.clone().unwrap().public_key;

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
