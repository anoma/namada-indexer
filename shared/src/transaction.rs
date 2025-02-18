use std::collections::HashMap;
use std::fmt::Display;

use namada_governance::{InitProposalData, VoteProposalData};
use namada_sdk::address::Address;
use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::events::extend::MaspTxRef;
use namada_sdk::key::common::PublicKey;
use namada_sdk::token::Transfer;
use namada_sdk::uint::Uint;
use namada_tx::data::pos::{
    BecomeValidator, Bond, ClaimRewards, CommissionChange, MetaDataChange,
    Redelegation, Unbond, Withdraw,
};
use namada_tx::data::{TxType, compute_inner_tx_hash};
use namada_tx::either::Either;
use namada_tx::{Section, Tx};
use serde::Serialize;

use crate::block::BlockHeight;
use crate::block_result::{BlockResult, TxEventStatusCode};
use crate::checksums::Checksums;
use crate::id::Id;
use crate::ser::{IbcMessage, TransferData};
use crate::utils::{self, transfer_to_ibc_tx_kind};

// We wrap public key in a struct so we serialize data as object instead of
// string
#[derive(Serialize, Debug, Clone)]
pub struct RevealPkData {
    pub public_key: PublicKey,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TransactionKind {
    TransparentTransfer(Option<TransferData>),
    ShieldedTransfer(Option<TransferData>),
    ShieldingTransfer(Option<TransferData>),
    UnshieldingTransfer(Option<TransferData>),
    MixedTransfer(Option<TransferData>),
    /// Generic, non-transfer, IBC messages
    IbcMsg(Option<IbcMessage<Transfer>>),
    IbcTrasparentTransfer((crate::token::Token, TransferData)),
    IbcShieldingTransfer((crate::token::Token, TransferData)),
    IbcUnshieldingTransfer((crate::token::Token, TransferData)),
    Bond(Option<Bond>),
    Redelegation(Option<Redelegation>),
    Unbond(Option<Unbond>),
    Withdraw(Option<Withdraw>),
    ClaimRewards(Option<ClaimRewards>),
    ProposalVote(Option<VoteProposalData>),
    InitProposal(Option<InitProposalData>),
    MetadataChange(Option<MetaDataChange>),
    CommissionChange(Option<CommissionChange>),
    RevealPk(Option<RevealPkData>),
    BecomeValidator(Option<Box<BecomeValidator>>),
    ReactivateValidator(Option<Address>),
    DeactivateValidator(Option<Address>),
    UnjailValidator(Option<Address>),
    Unknown,
}

impl TransactionKind {
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(&self).ok()
    }

    pub fn from(
        tx_kind_name: &str,
        data: &[u8],
        native_token: Address,
    ) -> Self {
        match tx_kind_name {
            "tx_transfer" => {
                if let Ok(transfer) = Transfer::try_from_slice(data) {
                    utils::transfer_to_tx_kind(transfer)
                } else {
                    TransactionKind::Unknown
                }
            }
            "tx_bond" => {
                let data = if let Ok(data) = Bond::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::Bond(data)
            }
            "tx_redelegate" => {
                let data = if let Ok(data) = Redelegation::try_from_slice(data)
                {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::Redelegation(data)
            }
            "tx_unbond" => {
                let data = if let Ok(data) = Unbond::try_from_slice(data) {
                    Some(Unbond::from(data))
                } else {
                    None
                };
                TransactionKind::Unbond(data)
            }
            "tx_withdraw" => {
                let data = if let Ok(data) = Withdraw::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::Withdraw(data)
            }
            "tx_claim_rewards" => {
                let data = if let Ok(data) = ClaimRewards::try_from_slice(data)
                {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::ClaimRewards(data)
            }
            "tx_init_proposal" => {
                let data =
                    if let Ok(data) = InitProposalData::try_from_slice(data) {
                        Some(data)
                    } else {
                        None
                    };
                TransactionKind::InitProposal(data)
            }
            "tx_vote_proposal" => {
                let data =
                    if let Ok(data) = VoteProposalData::try_from_slice(data) {
                        Some(data)
                    } else {
                        None
                    };
                TransactionKind::ProposalVote(data)
            }
            "tx_change_validator_metadata" => {
                let data =
                    if let Ok(data) = MetaDataChange::try_from_slice(data) {
                        Some(data)
                    } else {
                        None
                    };
                TransactionKind::MetadataChange(data)
            }
            "tx_commission_change" => {
                let data =
                    if let Ok(data) = CommissionChange::try_from_slice(data) {
                        Some(data)
                    } else {
                        None
                    };
                TransactionKind::CommissionChange(data)
            }
            "tx_reveal_pk" => {
                let data = if let Ok(data) = PublicKey::try_from_slice(data) {
                    Some(RevealPkData { public_key: data })
                } else {
                    None
                };
                TransactionKind::RevealPk(data)
            }
            "tx_deactivate_validator" => {
                let data = if let Ok(data) = Address::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::DeactivateValidator(data)
            }
            "tx_reactivate_validator" => {
                let data = if let Ok(data) = Address::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::ReactivateValidator(data)
            }
            "tx_ibc" => {
                if let Ok(ibc_data) =
                    namada_ibc::decode_message::<Transfer>(data)
                {
                    transfer_to_ibc_tx_kind(ibc_data, native_token)
                } else {
                    tracing::warn!("Cannot deserialize IBC transaction");
                    TransactionKind::IbcMsg(None)
                }
            }
            "tx_unjail_validator" => {
                let data = if let Ok(data) = Address::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                TransactionKind::UnjailValidator(data)
            }
            "tx_become_validator" => {
                let data =
                    if let Ok(data) = BecomeValidator::try_from_slice(data) {
                        Some(data)
                    } else {
                        None
                    };
                TransactionKind::BecomeValidator(data.map(Box::new))
            }
            _ => {
                tracing::warn!("Unknown transaction kind: {}", tx_kind_name);
                TransactionKind::Unknown
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionExitStatus {
    Applied,
    Rejected,
}

impl Display for TransactionExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Applied => write!(f, "Applied"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

impl From<TxEventStatusCode> for TransactionExitStatus {
    fn from(value: TxEventStatusCode) -> Self {
        match value {
            TxEventStatusCode::Ok => Self::Applied,
            TxEventStatusCode::Fail => Self::Rejected,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub inner_hash: Option<Id>,
    pub kind: TransactionKind,
    pub extra_sections: HashMap<Id, Vec<u8>>,
    pub index: usize,
    pub memo: Option<Vec<u8>>,
    pub fee: Fee,
}

pub struct MaspSectionData {
    pub total_notes: u64,
}

#[derive(Debug, Clone)]
pub struct Transaction2 {
    pub wrapper: WrapperTransaction,
    pub inners: InnerTransaction,
}

#[derive(Debug, Clone)]
pub struct WrapperTransaction {
    pub tx_id: Id,
    pub index: usize,
    pub fee: Fee,
    pub atomic: bool,
    pub block_height: BlockHeight,
    pub exit_code: TransactionExitStatus,
    pub total_signatures: u64,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct InnerTransaction {
    pub tx_id: Id,
    pub index: usize,
    pub wrapper_id: Id,
    pub kind: TransactionKind,
    pub memo: Option<String>,
    pub data: Option<String>,
    pub extra_sections: HashMap<Id, Vec<u8>>,
    pub notes: u64,
    pub exit_code: TransactionExitStatus,
}

impl InnerTransaction {
    pub fn get_section_data_by_id(&self, section_id: Id) -> Option<Vec<u8>> {
        self.extra_sections.get(&section_id).cloned()
    }

    /// An inner transaction is successful only if both the inner tx itself and
    /// the containing wrapper are marked as applied
    pub fn was_successful(&self, wrapper_tx: &WrapperTransaction) -> bool {
        wrapper_tx.exit_code == TransactionExitStatus::Applied
            && self.exit_code == TransactionExitStatus::Applied
    }

    pub fn is_ibc(&self) -> bool {
        matches!(
            self.kind,
            TransactionKind::IbcMsg(_)
                | TransactionKind::IbcTrasparentTransfer(_)
                | TransactionKind::IbcUnshieldingTransfer(_)
                | TransactionKind::IbcShieldingTransfer(_)
        )
    }
}

#[derive(Debug, Clone)]
pub struct Fee {
    pub gas: String,
    pub gas_used: Option<u64>,
    pub amount_per_gas_unit: String,
    pub gas_payer: Id,
    pub gas_token: Id,
}

impl Transaction {
    pub fn deserialize(
        raw_tx_bytes: &[u8],
        index: usize,
        block_height: BlockHeight,
        checksums: Checksums,
        block_results: &BlockResult,
        native_token: &Address,
    ) -> Result<(WrapperTransaction, Vec<InnerTransaction>), String> {
        let transaction =
            Tx::try_from(raw_tx_bytes).map_err(|e| e.to_string())?;
        let total_signatures = transaction
            .clone()
            .sections
            .iter()
            .filter(|section| section.signature().is_some())
            .count() as u64;
        let tx_size = raw_tx_bytes.len() as u64;

        match transaction.header().tx_type {
            TxType::Wrapper(wrapper) => {
                let wrapper_tx_id = Id::from(transaction.header_hash());
                let wrapper_tx_status =
                    block_results.is_wrapper_tx_applied(&wrapper_tx_id);
                let gas_used = block_results
                    .gas_used(&wrapper_tx_id)
                    .map(|gas| gas.parse::<u64>().unwrap());
                let mut masp_refs =
                    block_results.masp_refs(&wrapper_tx_id, index as u64);

                let fee = Fee {
                    gas: Uint::from(wrapper.gas_limit).to_string(),
                    gas_used,
                    amount_per_gas_unit: wrapper
                        .fee
                        .amount_per_gas_unit
                        .to_string_precise(),
                    gas_payer: Id::from(wrapper.fee_payer()),
                    gas_token: Id::from(wrapper.fee.token),
                };

                let atomic = transaction.header().atomic;

                let wrapper_tx = WrapperTransaction {
                    tx_id: wrapper_tx_id.clone(),
                    index,
                    fee,
                    atomic,
                    block_height,
                    exit_code: wrapper_tx_status.clone(),
                    total_signatures,
                    size: tx_size,
                };

                let mut inner_txs = vec![];

                for (index, tx_commitment) in
                    transaction.header().batch.into_iter().enumerate()
                {
                    let inner_tx_id = Id::from(compute_inner_tx_hash(
                        Some(&transaction.header_hash()),
                        Either::Right(&tx_commitment),
                    ));

                    let memo =
                        transaction.memo(&tx_commitment).map(|memo_bytes| {
                            String::from_utf8_lossy(
                                &subtle_encoding::hex::encode(memo_bytes),
                            )
                            .to_string()
                        });

                    let tx_code_id = transaction
                        .get_section(tx_commitment.code_sechash())
                        .and_then(|s| s.code_sec())
                        .map(|s| s.code.hash().0)
                        .map(|bytes| {
                            String::from_utf8(subtle_encoding::hex::encode(
                                bytes,
                            ))
                            .unwrap()
                        });

                    let tx_data =
                        transaction.data(&tx_commitment).unwrap_or_default();

                    let tx_kind = if let Some(id) = tx_code_id {
                        if let Some(tx_kind_name) =
                            checksums.get_name_by_id(&id)
                        {
                            TransactionKind::from(
                                &tx_kind_name,
                                &tx_data,
                                native_token.to_owned(),
                            )
                        } else {
                            TransactionKind::Unknown
                        }
                    } else {
                        TransactionKind::Unknown
                    };

                    let encoded_tx_data = if !tx_data.is_empty() {
                        tx_kind.to_json()
                    } else {
                        None
                    };

                    let inner_tx_status = block_results
                        .is_inner_tx_accepted(&wrapper_tx_id, &inner_tx_id);

                    let extra_sections = transaction
                        .sections
                        .iter()
                        .filter_map(|section| match section {
                            Section::ExtraData(code) => match code.code.clone()
                            {
                                namada_tx::Commitment::Hash(_) => None,
                                namada_tx::Commitment::Id(data) => {
                                    Some((Id::from(section.get_hash()), data))
                                }
                            },
                            _ => None,
                        })
                        .fold(HashMap::new(), |mut acc, (id, data)| {
                            acc.insert(id, data);
                            acc
                        });

                    // MASP events are emitted only for successful inner txs
                    let masp_bundle = matches!(
                        (&wrapper_tx_status, &inner_tx_status),
                        (
                            &TransactionExitStatus::Applied,
                            &TransactionExitStatus::Applied
                        )
                    )
                    .then(|| {
                        if let Some(note) = masp_refs.0.first() {
                            {
                                // Check if the masp ref is pointing to this
                                // inner tx
                                match &tx_kind {
                                    TransactionKind::ShieldedTransfer(
                                        Some(data),
                                    )
                                    | TransactionKind::ShieldingTransfer(
                                        Some(data),
                                    )
                                    | TransactionKind::UnshieldingTransfer(
                                        Some(data),
                                    )
                                    | TransactionKind::MixedTransfer(Some(
                                        data,
                                    )) => data.shielded_section_hash.and_then(
                                        |shielded_section_hash| {
                                            extract_masp_transaction(
                                                &transaction,
                                                note,
                                                &MaspTxRef::MaspSection(
                                                    shielded_section_hash,
                                                ),
                                            )
                                        },
                                    ),
                                    TransactionKind::IbcShieldingTransfer(
                                        (_, _),
                                    ) => extract_masp_transaction(
                                        &transaction,
                                        note,
                                        &MaspTxRef::IbcData(
                                            tx_commitment.data_hash,
                                        ),
                                    ),
                                    TransactionKind::IbcUnshieldingTransfer(
                                        (_, data),
                                    ) => data.shielded_section_hash.and_then(
                                        |shielded_section_hash| {
                                            extract_masp_transaction(
                                                &transaction,
                                                note,
                                                &MaspTxRef::MaspSection(
                                                    shielded_section_hash,
                                                ),
                                            )
                                        },
                                    ),
                                    _ => None,
                                }
                            }
                        } else {
                            None
                        }
                    })
                    .flatten();

                    let notes = masp_bundle.map_or(0, |bundle| {
                        // Remove the masp ref from the collection if we
                        // found one
                        masp_refs.0.remove(0);
                        bundle.sapling_bundle().map_or(0, |bundle| {
                            bundle.shielded_spends.len()
                                + bundle.shielded_outputs.len()
                                + bundle.shielded_converts.len()
                        })
                    }) as u64;

                    let inner_tx = InnerTransaction {
                        tx_id: inner_tx_id,
                        index,
                        wrapper_id: wrapper_tx_id.clone(),
                        memo,
                        data: encoded_tx_data,
                        extra_sections,
                        notes,
                        exit_code: inner_tx_status,
                        kind: tx_kind,
                    };

                    inner_txs.push(inner_tx);
                }

                if !masp_refs.0.is_empty() {
                    return Err("Not all the MASP references have been \
                                indexed"
                        .to_string());
                }

                Ok((wrapper_tx, inner_txs))
            }
            TxType::Raw => {
                Err("Raw transaction are not supported.".to_string())
            }
            TxType::Protocol(_) => {
                Err("Protocol transaction are not supported.".to_string())
            }
        }
    }

    pub fn get_section_data_by_id(&self, section_id: Id) -> Option<Vec<u8>> {
        self.extra_sections.get(&section_id).cloned()
    }
}

// Check if the masp reference in the event matches this inner transaction and,
// if so, extract the relative masp data
fn extract_masp_transaction(
    tx: &Tx,
    event_masp_ref: &MaspTxRef,
    tx_masp_ref: &MaspTxRef,
) -> Option<namada_core::masp::MaspTransaction> {
    match (event_masp_ref, tx_masp_ref) {
        (MaspTxRef::MaspSection(masp_id), MaspTxRef::MaspSection(tx_id))
            if masp_id == tx_id =>
        {
            tx.get_masp_section(masp_id).cloned()
        }
        (MaspTxRef::IbcData(event_hash), MaspTxRef::IbcData(tx_hash))
            if event_hash == tx_hash =>
        {
            tx.get_data_section(event_hash).and_then(|section| {
                match namada_sdk::ibc::decode_message::<Transfer>(&section) {
                    Ok(namada_ibc::IbcMessage::Envelope(msg_envelope)) => {
                        namada_sdk::ibc::extract_masp_tx_from_envelope(
                            &msg_envelope,
                        )
                    }
                    _ => None,
                }
            })
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct IbcSequence {
    pub sequence_number: String,
    pub source_port: String,
    pub dest_port: String,
    pub source_channel: String,
    pub dest_channel: String,
    pub timeout: u64,
    pub tx_id: Id,
}

impl IbcSequence {
    pub fn id(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            self.dest_port,
            self.dest_channel,
            self.source_port,
            self.source_channel,
            self.sequence_number
        )
    }
}

#[derive(Debug, Clone)]
pub enum IbcAckStatus {
    Success,
    Fail,
    Timeout,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct IbcAck {
    pub sequence_number: String,
    pub source_port: String,
    pub dest_port: String,
    pub source_channel: String,
    pub dest_channel: String,
    pub status: IbcAckStatus,
}

impl IbcAck {
    pub fn id_source(&self) -> String {
        format!(
            "{}/{}/{}",
            self.source_port, self.source_channel, self.sequence_number
        )
    }

    pub fn id_dest(&self) -> String {
        format!(
            "{}/{}/{}",
            self.dest_port, self.dest_channel, self.sequence_number
        )
    }

    pub fn id(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            self.dest_port,
            self.dest_channel,
            self.source_port,
            self.source_channel,
            self.sequence_number
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TransactionHistoryKind {
    Received,
    Sent,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TransactionTarget {
    pub inner_tx: Id,
    pub address: String,
    pub kind: TransactionHistoryKind,
}

impl TransactionTarget {
    pub fn new(
        inner_tx: Id,
        address: String,
        kind: TransactionHistoryKind,
    ) -> Self {
        Self {
            inner_tx,
            address,
            kind,
        }
    }

    pub fn sent(inner_tx: Id, address: String) -> Self {
        Self::new(inner_tx, address, TransactionHistoryKind::Sent)
    }

    pub fn received(inner_tx: Id, address: String) -> Self {
        Self::new(inner_tx, address, TransactionHistoryKind::Received)
    }
}
