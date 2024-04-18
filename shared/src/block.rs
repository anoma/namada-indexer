use std::{collections::BTreeMap, str::FromStr};

use namada_sdk::address::Address;
use namada_sdk::borsh::BorshDeserialize;
use namada_tx::data::TxType;
use namada_tx::Tx;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;

use crate::checksums::Checksums;
use crate::header::BlockHeader;
use crate::id::Id;

pub type Epoch = u32;
pub type BlockHeight = u32;

#[derive(Debug, Clone)]
pub enum TransactionKind {
    Wrapper,
    Protocol,
    TransparentTransfer(Vec<u8>),
    ShieldedTransfer(Vec<u8>),
    Bond(Vec<u8>),
    Redelegation(Vec<u8>),
    Unbond(Vec<u8>),
    Withdraw(Vec<u8>),
    ClaimRewards(Vec<u8>),
    ReactivateValidator(Vec<u8>),
    DeactivateValidator(Vec<u8>),
    IbcEnvelop(Vec<u8>),
    IbcTransparentTransfer(Vec<u8>),
    IbcShieldedTransfer(Vec<u8>),
    ChangeConsensusKey(Vec<u8>),
    ChangeCommission(Vec<u8>),
    ChangeMetadata(Vec<u8>),
    BecomeValidator(Vec<u8>),
    InitAccount(Vec<u8>),
    InitProposal(Vec<u8>),
    ResignSteward(Vec<u8>),
    RevealPublicKey(Vec<u8>),
    UnjailValidator(Vec<u8>),
    UpdateAccount(Vec<u8>),
    UpdateStewardCommissions(Vec<u8>),
    ProposalVote(Vec<u8>),
    Unknown,
}

impl TransactionKind {
    pub fn from(tx_kind_name: &str, data: &[u8]) -> Self {
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
            _ => TransactionKind::Unknown,
        }
    }
}

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

// TODO: add later
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum TransactionExitStatus {
//     Accepted,
//     Applied,
//     Rejected,
// }

// impl Display for TransactionExitStatus {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Accepted => write!(f, "Accepted"),
//             Self::Applied => write!(f, "Applied"),
//             Self::Rejected => write!(f, "Rejected"),
//         }
//     }
// }

// impl TransactionExitStatus {
//     pub fn from(
//         tx_attributes: &TxAttributes,
//         tx_kind: &TransactionKind,
//     ) -> Self {
//         match (tx_kind, tx_attributes.code) {
//             (TransactionKind::Wrapper, TxEventStatusCode::Ok) => {
//                 TransactionExitStatus::Accepted
//             }
//             (_, TxEventStatusCode::Ok) => TransactionExitStatus::Applied,
//             (_, TxEventStatusCode::Fail) => TransactionExitStatus::Rejected,
//         }
//     }
// }

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RawMemo(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: Id,
    pub inner_hash: Option<Id>,
    pub kind: TransactionKind,
    pub index: usize,
}

impl Transaction {
    pub fn deserialize(
        raw_tx_bytes: &[u8],
        index: usize,
        checksums: Checksums,
    ) -> Result<(Self, String), String> {
        let transaction =
            Tx::try_from(raw_tx_bytes).map_err(|e| e.to_string())?;

        match transaction.header().tx_type {
            TxType::Wrapper(_) => {
                let tx_id = Id::from(transaction.header_hash());
                let raw_hash = Id::from(transaction.raw_header_hash());
                let raw_hash_str = raw_hash.to_string();

                let tx_code_id = transaction
                    .get_section(transaction.code_sechash())
                    .and_then(|s| s.code_sec())
                    .map(|s| s.code.hash().0)
                    .map(|bytes| {
                        String::from_utf8(subtle_encoding::hex::encode(bytes))
                            .unwrap()
                    });

                let tx_kind = if let Some(id) = tx_code_id {
                    if let Some(tx_kind_name) = checksums.get_name_by_id(&id) {
                        let tx_data = transaction.data().unwrap_or_default();
                        TransactionKind::from(&tx_kind_name, &tx_data)
                    } else {
                        TransactionKind::Unknown
                    }
                } else {
                    TransactionKind::Unknown
                };

                let transaction = Transaction {
                    hash: tx_id,
                    inner_hash: Some(raw_hash),
                    kind: tx_kind,
                    index,
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
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub epoch: Epoch,
}

impl Block {
    pub fn from(
        response: TendermintBlockResponse,
        checksums: Checksums,
        epoch: Epoch,
    ) -> Self {
        let transactions = response
            .block
            .data
            .iter()
            .enumerate()
            .filter_map(|(index, tx_raw_bytes)| {
                Transaction::deserialize(tx_raw_bytes, index, checksums.clone())
                    .map_err(|reason| {
                        tracing::info!(
                            "Couldn't deserialize tx due to {}",
                            reason
                        );
                    })
                    .ok()
                    .and_then(|(tx, _inner_hash)| {
                        if matches!(&tx.kind, TransactionKind::Unknown) {
                            return None;
                        }
                        // NB: skip tx if no memo is present

                        Some(tx)
                    })
            })
            .collect::<Vec<Transaction>>();

        Block {
            hash: Id::from(response.block_id.hash),
            header: BlockHeader {
                height: response.block.header.height.value() as BlockHeight,
                proposer_address: response
                    .block
                    .header
                    .proposer_address
                    .to_string()
                    .to_lowercase(),
                timestamp: response.block.header.time.to_string(),
                app_hash: Id::from(response.block.header.app_hash),
            },
            transactions,
            epoch,
        }
    }

    pub fn get_transfer_addresses(&self) -> Vec<Address> {
        self.transactions
            .iter()
            .filter_map(|tx| match &tx.kind {
                TransactionKind::TransparentTransfer(data) => {
                    let transfer_data =
                        namada_core::token::Transfer::try_from_slice(data)
                            .unwrap();
                    Some(vec![transfer_data.source, transfer_data.target])
                }
                _ => None,
            })
            .flatten()
            .collect()
    }
}
