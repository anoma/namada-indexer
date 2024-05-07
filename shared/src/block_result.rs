use std::{collections::BTreeMap, str::FromStr};
use tendermint_rpc::endpoint::block_results::Response as TendermintBlockResultResponse;

use crate::id::Id;

#[derive(Debug, Clone)]
pub enum EventKind {
    Applied,
    Rejected,
    Accepted,
    Unknown,
}

impl From<&String> for EventKind {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "applied" => Self::Applied,
            "accepted" => Self::Accepted,
            "rejected" => Self::Rejected,
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
    pub attributes: TxAttributes,
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
    pub fn deserialize(event_kind: &EventKind, attributes: &BTreeMap<String, String>) -> Self {
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

impl From<TendermintBlockResultResponse> for BlockResult {
    fn from(value: TendermintBlockResultResponse) -> Self {
        let begin_events = value
            .begin_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes =
                    event
                        .attributes
                        .iter()
                        .fold(BTreeMap::default(), |mut acc, attribute| {
                            acc.insert(attribute.key.clone(), attribute.value.clone());
                            acc
                        });
                let attributes = TxAttributes::deserialize(&kind, &raw_attributes);
                Event { kind, attributes }
            })
            .collect::<Vec<Event>>();
        let end_events = value
            .end_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes =
                    event
                        .attributes
                        .iter()
                        .fold(BTreeMap::default(), |mut acc, attribute| {
                            acc.insert(attribute.key.clone(), attribute.value.clone());
                            acc
                        });
                let attributes = TxAttributes::deserialize(&kind, &raw_attributes);
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
    pub fn find_tx_hash_result(&self, tx_hash: &Id) -> Option<TxAttributes> {
        self.end_events
            .iter()
            .find(|event| event.attributes.hash.eq(tx_hash))
            .map(|event| event.attributes.clone())
    }
}