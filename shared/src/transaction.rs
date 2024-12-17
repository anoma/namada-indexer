use std::collections::HashMap;
use std::fmt::Display;

use namada_governance::{InitProposalData, VoteProposalData};
use namada_sdk::address::Address;
use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::key::common::PublicKey;
use namada_sdk::masp::ShieldedTransfer;
use namada_sdk::token::Transfer;
use namada_sdk::uint::Uint;
use namada_tx::data::pos::{
    BecomeValidator, Bond, ClaimRewards, CommissionChange, MetaDataChange,
    Redelegation, Unbond, Withdraw,
};
use namada_tx::data::{compute_inner_tx_hash, TxType};
use namada_tx::either::Either;
use namada_tx::{Section, Tx};
use serde::Serialize;

use crate::block::BlockHeight;
use crate::block_result::{BlockResult, TxEventStatusCode};
use crate::checksums::Checksums;
use crate::id::Id;
use crate::ser::{IbcMessage, TransparentTransfer};

// We wrap public key in a struct so we serialize data as object instead of
// string
#[derive(Serialize, Debug, Clone)]
pub struct RevealPkData {
    pub public_key: PublicKey,
}

// Capture details for unknown transactions so we can store them in the db
#[derive(Serialize, Debug, Clone)]
pub struct UnknownTransaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_optional_bytes_to_hex")]
    pub data: Option<Vec<u8>>,
}

fn serialize_optional_bytes_to_hex<S>(
    bytes: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    bytes
        .as_ref()
        .map(|b| {
            let mut s = String::with_capacity(2 + (b.len() * 2)); // "0x" + 2 chars per byte
            s.push_str("0x");
            for byte in b {
                use std::fmt::Write;
                write!(s, "{:02x}", byte).unwrap();
            }
            s
        })
        .serialize(serializer)
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TransactionKind {
    TransparentTransfer(Option<TransparentTransfer>),
    // TODO: remove once ShieldedTransfer can be serialized
    #[serde(skip)]
    ShieldedTransfer(Option<ShieldedTransfer>),
    IbcMsgTransfer(Option<IbcMessage<Transfer>>),
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
    Unknown(Option<UnknownTransaction>),
}

impl TransactionKind {
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(&self).ok()
    }

    pub fn from(id: &str, tx_kind_name: &str, data: &[u8]) -> Self {
        match tx_kind_name {
            "tx_transfer" => {
                let data = if let Ok(data) = Transfer::try_from_slice(data) {
                    Some(TransparentTransfer::from(data))
                } else {
                    None
                };
                TransactionKind::TransparentTransfer(data)
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
                let data = if let Ok(data) =
                    namada_ibc::decode_message::<Transfer>(data)
                {
                    Some(data)
                } else {
                    tracing::warn!("Cannot deserialize IBC transfer");
                    None
                };
                TransactionKind::IbcMsgTransfer(data.map(IbcMessage))
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
                TransactionKind::Unknown(Some(UnknownTransaction {
                    id: Some(id.to_string()),
                    name: Some(tx_kind_name.to_string()),
                    data: Some(data.to_vec()),
                }))
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

#[derive(Debug, Clone)]
pub struct WrapperTransaction {
    pub tx_id: Id,
    pub index: usize,
    pub fee: Fee,
    pub atomic: bool,
    pub block_height: BlockHeight,
    pub exit_code: TransactionExitStatus,
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
    pub exit_code: TransactionExitStatus,
}

impl InnerTransaction {
    pub fn get_section_data_by_id(&self, section_id: Id) -> Option<Vec<u8>> {
        self.extra_sections.get(&section_id).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct Fee {
    pub gas: String,
    pub gas_used: Option<String>,
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
    ) -> Result<(WrapperTransaction, Vec<InnerTransaction>), String> {
        let transaction =
            Tx::try_from(raw_tx_bytes).map_err(|e| e.to_string())?;

        match transaction.header().tx_type {
            TxType::Wrapper(wrapper) => {
                let wrapper_tx_id = Id::from(transaction.header_hash());
                let wrapper_tx_status =
                    block_results.is_wrapper_tx_applied(&wrapper_tx_id);
                let gas_used = block_results.gas_used(&wrapper_tx_id);

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
                    exit_code: wrapper_tx_status,
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
                            TransactionKind::from(&id, &tx_kind_name, &tx_data)
                        } else {
                            TransactionKind::Unknown(Some(UnknownTransaction {
                                id: Some(id),
                                name: None,
                                data: Some(tx_data.clone()),
                            }))
                        }
                    } else {
                        TransactionKind::Unknown(Some(UnknownTransaction {
                            id: None,
                            name: None,
                            data: Some(tx_data.clone()),
                        }))
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

                    let inner_tx = InnerTransaction {
                        tx_id: inner_tx_id,
                        index,
                        wrapper_id: wrapper_tx_id.clone(),
                        memo,
                        data: encoded_tx_data,
                        extra_sections,
                        exit_code: inner_tx_status,
                        kind: tx_kind,
                    };

                    inner_txs.push(inner_tx);
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
