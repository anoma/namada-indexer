use std::collections::HashMap;
use std::fmt::Display;

use namada_governance::{InitProposalData, VoteProposalData};
use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::key::common::PublicKey;
use namada_sdk::masp::ShieldedTransfer;
use namada_sdk::token::Transfer;
use namada_sdk::uint::Uint;
use namada_tx::data::pos::{
    Bond, ClaimRewards, CommissionChange, MetaDataChange, Redelegation, Unbond,
    Withdraw,
};
use namada_tx::data::TxType;
use namada_tx::{Section, Tx};
use serde::Serialize;

use crate::block::BlockHeight;
use crate::block_result::{BlockResult, TxEventStatusCode};
use crate::checksums::Checksums;
use crate::id::Id;
use crate::ser::TransparentTransfer;

// We wrap public key in a struct so we serialize data as object instead of
// string
#[derive(Serialize, Debug, Clone)]
pub struct RevealPkData {
    pub public_key: PublicKey,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TransactionKind {
    TransparentTransfer(TransparentTransfer),
    // TODO: remove once ShieldedTransfer can be serialized
    #[serde(skip)]
    ShieldedTransfer(ShieldedTransfer),
    Bond(Bond),
    Redelegation(Redelegation),
    Unbond(Unbond),
    Withdraw(Withdraw),
    ClaimRewards(ClaimRewards),
    ProposalVote(VoteProposalData),
    InitProposal(InitProposalData),
    MetadataChange(MetaDataChange),
    CommissionChange(CommissionChange),
    RevealPk(RevealPkData),
    Unknown,
}

impl TransactionKind {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Cannot serialize TransactionKind")
    }

    pub fn from(tx_kind_name: &str, data: &[u8]) -> Self {
        match tx_kind_name {
            "tx_transfer" => {
                TransactionKind::TransparentTransfer(TransparentTransfer::from(
                    Transfer::try_from_slice(data)
                        .expect("Cannot deserialize Transfer"),
                ))
            }
            "tx_bond" => TransactionKind::Bond(
                Bond::try_from_slice(data).expect("Cannot deserialize Bond"),
            ),
            "tx_redelegation" => TransactionKind::Redelegation(
                Redelegation::try_from_slice(data)
                    .expect("Cannot deserialize Redelegation"),
            ),
            "tx_unbond" => TransactionKind::Unbond(
                Unbond::try_from_slice(data)
                    .expect("Cannot deserialize Unbond"),
            ),
            "tx_withdraw" => TransactionKind::Withdraw(
                Withdraw::try_from_slice(data)
                    .expect("Cannot deserialize Withdraw"),
            ),
            "tx_claim_rewards" => TransactionKind::ClaimRewards(
                ClaimRewards::try_from_slice(data)
                    .expect("Cannot deserialize ClaimRewards"),
            ),
            "tx_init_proposal" => TransactionKind::InitProposal(
                InitProposalData::try_from_slice(data)
                    .expect("Cannot deserialize InitProposal"),
            ),
            "tx_vote_proposal" => TransactionKind::ProposalVote(
                VoteProposalData::try_from_slice(data)
                    .expect("Cannot deserialize VoteProposal"),
            ),
            "tx_metadata_change" => TransactionKind::MetadataChange(
                MetaDataChange::try_from_slice(data)
                    .expect("Cannot deserialize MetaDataChange"),
            ),
            "tx_commission_change" => TransactionKind::CommissionChange(
                CommissionChange::try_from_slice(data)
                    .expect("Cannot deserialize CommissionChange"),
            ),
            "tx_reveal_pk" => TransactionKind::RevealPk(RevealPkData {
                public_key: PublicKey::try_from_slice(data)
                    .expect("Cannot deserialize PublicKey"),
            }),
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
                        Some(tx_kind.to_json())
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
