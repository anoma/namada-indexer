use std::collections::HashMap;
use std::fmt::Display;

use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::uint::Uint;
use namada_tx::data::TxType;
use namada_tx::{Section, Tx};

use crate::block_result::{BlockResult, TxAttributes, TxEventStatusCode};
use crate::checksums::Checksums;
use crate::id::Id;

#[derive(Debug, Clone)]
pub enum TransactionKind {
    Wrapper,
    TransparentTransfer(Vec<u8>),
    ShieldedTransfer(Vec<u8>),
    Bond(Vec<u8>),
    Redelegation(Vec<u8>),
    Unbond(Vec<u8>),
    Withdraw(Vec<u8>),
    ClaimRewards(Vec<u8>),
    ProposalVote(Vec<u8>),
    InitProposal(Vec<u8>),
    MetadataChange(Vec<u8>),
    Unknown,
}

impl TransactionKind {
    pub fn from(tx_kind_name: &str, data: &[u8]) -> Self {
        println!("tx kind: {}", tx_kind_name);
        match tx_kind_name {
            "tx_transfer" => {
                let transfer_data =
                    namada_core::token::Transfer::try_from_slice(data).unwrap();
                match transfer_data.shielded {
                    Some(_) => TransactionKind::ShieldedTransfer(data.to_vec()),
                    None => TransactionKind::TransparentTransfer(data.to_vec()),
                }
            }
            "tx_bond" => TransactionKind::Bond(data.to_vec()),
            "tx_redelegation" => TransactionKind::Redelegation(data.to_vec()),
            "tx_unbond" => TransactionKind::Unbond(data.to_vec()),
            "tx_withdraw" => TransactionKind::Withdraw(data.to_vec()),
            "tx_claim_rewards" => TransactionKind::ClaimRewards(data.to_vec()),
            "wrapper" => TransactionKind::Wrapper,
            "tx_init_proposal" => TransactionKind::InitProposal(data.to_vec()),
            "tx_vote_proposal" => TransactionKind::ProposalVote(data.to_vec()),
            "tx_metadata_change" => {
                TransactionKind::MetadataChange(data.to_vec())
            }
            _ => TransactionKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionExitStatus {
    Accepted,
    Applied,
    Rejected,
}

impl Display for TransactionExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted => write!(f, "Accepted"),
            Self::Applied => write!(f, "Applied"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

impl TransactionExitStatus {
    pub fn from(
        tx_attributes: &TxAttributes,
        tx_kind: &TransactionKind,
    ) -> Self {
        match (tx_kind, tx_attributes.code) {
            (TransactionKind::Wrapper, TxEventStatusCode::Ok) => {
                TransactionExitStatus::Accepted
            }
            (_, TxEventStatusCode::Ok) => TransactionExitStatus::Applied,
            (_, TxEventStatusCode::Fail) => TransactionExitStatus::Rejected,
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
pub struct Fee {
    pub gas: String,
    pub gas_payer: Id,
    pub gas_token: Id,
}

impl Transaction {
    pub fn deserialize(
        raw_tx_bytes: &[u8],
        index: usize,
        checksums: Checksums,
        block_results: &BlockResult,
    ) -> Result<(Self, String), String> {
        let transaction =
            Tx::try_from(raw_tx_bytes).map_err(|e| e.to_string())?;

        match transaction.header().tx_type {
            TxType::Wrapper(wrapper) => {
                let tx_id = Id::from(transaction.header_hash());
                let raw_hash = Id::from(transaction.raw_header_hash());
                let raw_hash_str = raw_hash.to_string();
                let wrapper_tx_status =
                    block_results.find_tx_hash_result(&tx_id).unwrap();
                let memo = transaction.memo();
                let fee = Fee {
                    gas: Uint::from(wrapper.gas_limit).to_string(),
                    gas_payer: Id::from(wrapper.pk),
                    gas_token: Id::from(wrapper.fee.token),
                };

                let wrapper_tx_exit = TransactionExitStatus::from(
                    &wrapper_tx_status,
                    &TransactionKind::Wrapper,
                );
                if wrapper_tx_exit == TransactionExitStatus::Rejected {
                    return Err(format!("Wrapper {} was rejected", tx_id));
                };

                let tx_status =
                    block_results.find_tx_hash_result(&tx_id).unwrap();
                let tx_exit = TransactionExitStatus::from(
                    &tx_status,
                    &TransactionKind::Wrapper,
                );
                if tx_exit == TransactionExitStatus::Rejected {
                    return Err(format!(
                        "Transaction {} was rejected",
                        raw_hash
                    ));
                };

                let tx_code_id = transaction
                    .get_section(transaction.code_sechash())
                    .and_then(|s| s.code_sec())
                    .map(|s| s.code.hash().0)
                    .map(|bytes| {
                        String::from_utf8(subtle_encoding::hex::encode(bytes))
                            .unwrap()
                    });

                let extra_sections = transaction
                    .sections
                    .iter()
                    .filter_map(|section| match section {
                        Section::ExtraData(code) => match code.code.clone() {
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

                let tx_kind = if let Some(id) = tx_code_id {
                    if let Some(tx_kind_name) = checksums.get_name_by_id(&id) {
                        let tx_data = transaction.data().unwrap_or_default();
                        TransactionKind::from(&tx_kind_name, &tx_data)
                    } else {
                        println!("UNKNOWN: {}", id);
                        TransactionKind::Unknown
                    }
                } else {
                    TransactionKind::Unknown
                };

                let transaction = Transaction {
                    hash: tx_id,
                    inner_hash: Some(raw_hash),
                    kind: tx_kind,
                    extra_sections,
                    index,
                    memo,
                    fee,
                };

                Ok((transaction, raw_hash_str))
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
