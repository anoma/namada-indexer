use std::{collections::BTreeMap, str::FromStr};

use namada_sdk::address::Address;
use namada_sdk::borsh::BorshDeserialize;
use tendermint_rpc::endpoint::block::Response as TendermintBlockResponse;

use crate::block_result::BlockResult;
use crate::checksums::Checksums;
use crate::header::BlockHeader;
use crate::id::Id;
use crate::transaction::{Transaction, TransactionKind};

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
    pub transactions: Vec<Transaction>,
    pub epoch: Epoch,
}

impl Block {
    pub fn from(
        block_response: TendermintBlockResponse,
        block_results: &BlockResult,
        checksums: Checksums,
        epoch: Epoch,
    ) -> Self {
        let transactions = block_response
            .block
            .data
            .iter()
            .enumerate()
            .filter_map(|(index, tx_raw_bytes)| {
                Transaction::deserialize(tx_raw_bytes, index, checksums.clone(), block_results)
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
            hash: Id::from(block_response.block_id.hash),
            header: BlockHeader {
                height: block_response.block.header.height.value() as BlockHeight,
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

    //TODO: this can be potentially optimized by removing duplicates
    pub fn addresses_with_balance_change(&self) -> Vec<Address> {
        self.transactions
            .iter()
            .filter_map(|tx| match &tx.kind {
                TransactionKind::TransparentTransfer(data) => {
                    let transfer_data =
                        namada_core::token::Transfer::try_from_slice(data)
                            .unwrap();
                    Some(vec![transfer_data.source, transfer_data.target])
                }
                TransactionKind::Bond(data) => {
                    let bond_data =
                        namada_tx::data::pos::Bond::try_from_slice(data)
                            .unwrap();
                    let address =
                        bond_data.source.unwrap_or(bond_data.validator);

                    Some(vec![address])
                }
                TransactionKind::Withdraw(data) => {
                    let withdraw_data =
                        namada_tx::data::pos::Withdraw::try_from_slice(data)
                            .unwrap();
                    let address =
                        withdraw_data.source.unwrap_or(withdraw_data.validator);

                    Some(vec![address])
                }
                TransactionKind::ClaimRewards(data) => {
                    let claim_rewards_data =
                        namada_tx::data::pos::ClaimRewards::try_from_slice(
                            data,
                        )
                        .unwrap();
                    let address = claim_rewards_data
                        .source
                        .unwrap_or(claim_rewards_data.validator);

                    Some(vec![address])
                }
                _ => None,
            })
            .flatten()
            .collect()
    }
}
