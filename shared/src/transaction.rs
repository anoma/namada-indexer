use std::collections::HashMap;
use std::fmt::Display;

use namada_sdk::uint::Uint;
use namada_tx::data::TxType;
use namada_tx::{Section, Tx};
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::{Rng, RngCore};

use crate::block::BlockHeight;
use crate::block_result::{BlockResult, TxEventStatusCode};
use crate::checksums::Checksums;
use crate::id::Id;

#[derive(Debug, Clone)]
pub enum TransactionKind {
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
    CommissionChange(Vec<u8>),
    RevealPk(Vec<u8>),
    Unknown,
}

impl TransactionKind {
    pub fn from(tx_kind_name: &str, data: &[u8]) -> Self {
        match tx_kind_name {
            "tx_transparent_transfer" => {
                TransactionKind::TransparentTransfer(data.to_vec())
            }
            "tx_bond" => TransactionKind::Bond(data.to_vec()),
            "tx_redelegation" => TransactionKind::Redelegation(data.to_vec()),
            "tx_unbond" => TransactionKind::Unbond(data.to_vec()),
            "tx_withdraw" => TransactionKind::Withdraw(data.to_vec()),
            "tx_claim_rewards" => TransactionKind::ClaimRewards(data.to_vec()),
            "tx_init_proposal" => TransactionKind::InitProposal(data.to_vec()),
            "tx_vote_proposal" => TransactionKind::ProposalVote(data.to_vec()),
            "tx_metadata_change" => {
                TransactionKind::MetadataChange(data.to_vec())
            }
            "tx_commission_change" => {
                TransactionKind::CommissionChange(data.to_vec())
            }
            "tx_reveal_pk" => TransactionKind::RevealPk(data.to_vec()),
            _ => TransactionKind::Unknown,
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
}

impl WrapperTransaction {
    pub fn fake() -> Self {
        let tx_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let tx_index = rand::thread_rng().gen_range(0..10);
        let gas_payer =
            namada_core::address::gen_established_address("namada-indexer");
        let gas_token =
            namada_core::address::gen_established_address("namada-indexer");

        Self {
            tx_id: Id::Hash(tx_id),
            index: tx_index,
            fee: Fee {
                gas: "20000".to_string(),
                amount_per_gas_unit: "0.00000".to_string(),
                gas_payer: Id::from(gas_payer),
                gas_token: Id::from(gas_token),
            },
            atomic: rand::random(),
            block_height: rand::random(),
            exit_code: rand::random(),
        }
    }
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
    pub fn fake(wrapper_id: Id) -> Self {
        let tx_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let tx_index = rand::thread_rng().gen_range(0..10);
        let tx_kind: TransactionKind = rand::random();

        let mut data = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut data);
        let memo = String::from_utf8_lossy(&subtle_encoding::hex::encode(data))
            .to_string();

        let tx_exit_code: TransactionExitStatus = rand::random();

        Self {
            tx_id: Id::Hash(tx_id),
            index: tx_index,
            wrapper_id,
            kind: tx_kind,
            memo: Some(memo),
            data: None,
            extra_sections: HashMap::new(),
            exit_code: tx_exit_code,
        }
    }

    pub fn get_section_data_by_id(&self, section_id: Id) -> Option<Vec<u8>> {
        self.extra_sections.get(&section_id).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct Fee {
    pub gas: String,
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

                let fee = Fee {
                    gas: Uint::from(wrapper.gas_limit).to_string(),
                    amount_per_gas_unit: wrapper
                        .fee
                        .amount_per_gas_unit
                        .to_string_precise(),
                    gas_payer: Id::from(wrapper.pk),
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
                    let inner_tx_id = Id::from(tx_commitment.get_hash());

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
                            TransactionKind::from(&tx_kind_name, &tx_data)
                        } else {
                            TransactionKind::Unknown
                        }
                    } else {
                        TransactionKind::Unknown
                    };

                    let encoded_tx_data = if !tx_data.is_empty() {
                        let hex_encode = subtle_encoding::hex::encode(tx_data);
                        let encoded_data = String::from_utf8_lossy(&hex_encode);
                        Some(encoded_data.to_string())
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

impl Distribution<TransactionKind> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> TransactionKind {
        match rng.gen_range(0..=11) {
            0 => TransactionKind::Bond(vec![]),
            1 => TransactionKind::ClaimRewards(vec![]),
            2 => TransactionKind::CommissionChange(vec![]),
            3 => TransactionKind::InitProposal(vec![]),
            4 => TransactionKind::MetadataChange(vec![]),
            5 => TransactionKind::ProposalVote(vec![]),
            6 => TransactionKind::Redelegation(vec![]),
            7 => TransactionKind::RevealPk(vec![]),
            8 => TransactionKind::ShieldedTransfer(vec![]),
            9 => TransactionKind::TransparentTransfer(vec![]),
            10 => TransactionKind::Unbond(vec![]),
            _ => TransactionKind::Withdraw(vec![]),
        }
    }
}

impl Distribution<TransactionExitStatus> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> TransactionExitStatus {
        match rng.gen_range(0..=1) {
            0 => TransactionExitStatus::Applied,
            _ => TransactionExitStatus::Rejected,
        }
    }
}
