use std::collections::BTreeMap;
use std::str::FromStr;

use namada_tx::data::TxResult;
use serde::Deserialize;
use tendermint_rpc::endpoint::block_results::Response as TendermintBlockResultResponse;

use crate::id::Id;
use crate::transaction::TransactionExitStatus;

#[derive(Debug, Clone)]
pub enum EventKind {
    Applied,
    Unknown,
}

impl From<&String> for EventKind {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "tx/applied" => Self::Applied,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockResult {
    pub height: u64,
    pub begin_events: Vec<Event>,
    pub end_events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    pub attributes: Option<TxAttributes>,
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
            "0" | "1" => Self::Ok,
            _ => Self::Fail,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BatchResults {
    pub batch_errors: BTreeMap<Id, BTreeMap<Id, String>>,
    pub batch_results: BTreeMap<Id, bool>,
}

impl From<TxResult<String>> for BatchResults {
    fn from(value: TxResult<String>) -> Self {
        Self {
            batch_results: value.batch_results.0.iter().fold(
                BTreeMap::default(),
                |mut acc, (tx_hash, result)| {
                    let tx_id = Id::from(*tx_hash);
                    let result = if let Ok(result) = result {
                        result.is_accepted()
                    } else {
                        false
                    };
                    acc.insert(tx_id, result);
                    acc
                },
            ),
            batch_errors: value.batch_results.0.into_iter().fold(
                BTreeMap::default(),
                |mut acc, (tx_hash, result)| {
                    let tx_id = Id::from(tx_hash);
                    let result = if let Ok(result) = result {
                        result
                            .vps_result
                            .errors
                            .into_iter()
                            .map(|(address, error)| (Id::from(address), error))
                            .collect()
                    } else {
                        BTreeMap::default()
                    };
                    acc.insert(tx_id, result);
                    acc
                },
            ),
        }
    }
}

impl BatchResults {
    pub fn is_successful(&self, tx_id: &Id) -> bool {
        match self.batch_results.get(tx_id) {
            Some(result) => *result,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TxAttributes {
    pub code: TxEventStatusCode,
    pub gas: u64,
    pub hash: Id,
    pub height: u64,
    pub batch: BatchResults,
    pub info: String,
}

impl TxAttributes {
    pub fn deserialize(
        event_kind: &EventKind,
        attributes: &BTreeMap<String, String>,
    ) -> Option<Self> {
        match event_kind {
            EventKind::Unknown => None,
            EventKind::Applied => Some(Self {
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
                batch: attributes
                    .get("batch")
                    .map(|batch_result| {
                        let tx_result: TxResult<String> =
                            serde_json::from_str(batch_result).unwrap();
                        BatchResults::from(tx_result)
                    })
                    .unwrap(),
                info: attributes.get("info").unwrap().to_owned(),
            }),
            _ => None,
        }
    }
}

impl From<TendermintBlockResultResponse> for BlockResult {
    fn from(value: TendermintBlockResultResponse) -> Self {
        let begin_events = value
            .begin_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes = event.attributes.iter().fold(
                    BTreeMap::default(),
                    |mut acc, attribute| {
                        acc.insert(
                            String::from(attribute.key_str().unwrap()),
                            String::from(attribute.value_str().unwrap()),
                        );
                        acc
                    },
                );
                let attributes =
                    TxAttributes::deserialize(&kind, &raw_attributes);
                Event { kind, attributes }
            })
            .collect::<Vec<Event>>();
        let end_events = value
            .end_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes = event.attributes.iter().fold(
                    BTreeMap::default(),
                    |mut acc, attribute| {
                        acc.insert(
                            String::from(attribute.key_str().unwrap()),
                            String::from(attribute.value_str().unwrap()),
                        );
                        acc
                    },
                );
                let attributes =
                    TxAttributes::deserialize(&kind, &raw_attributes);
                Event { kind, attributes }
            })
            .collect::<Vec<Event>>();
        Self {
            height: value.height.value(),
            begin_events,
            end_events,
        }
    }
}

impl From<&TendermintBlockResultResponse> for BlockResult {
    fn from(value: &TendermintBlockResultResponse) -> Self {
        BlockResult::from(value.clone())
    }
}

impl BlockResult {
    pub fn is_wrapper_tx_applied(&self, tx_hash: &Id) -> TransactionExitStatus {
        let exit_status = self
            .end_events
            .iter()
            .filter_map(|event| event.attributes.clone())
            .find(|attributes| attributes.hash.eq(tx_hash))
            .map(|attributes| attributes.clone().code)
            .map(TransactionExitStatus::from);

        exit_status.unwrap_or(TransactionExitStatus::Rejected)
    }

    pub fn is_inner_tx_accepted(
        &self,
        wrapper_hash: &Id,
        inner_hash: &Id,
    ) -> TransactionExitStatus {
        let exit_status = self
            .end_events
            .iter()
            .filter_map(|event| event.attributes.clone())
            .find(|attributes| attributes.hash.eq(wrapper_hash))
            .map(|attributes| attributes.batch.is_successful(inner_hash))
            .map(|successful| match successful {
                true => TransactionExitStatus::Applied,
                false => TransactionExitStatus::Rejected,
            });
        exit_status.unwrap_or(TransactionExitStatus::Rejected)
    }
}
