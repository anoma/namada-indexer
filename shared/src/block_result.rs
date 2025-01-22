use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use namada_sdk::events::extend::{IndexedMaspData, MaspTxRef};
use namada_tx::data::TxResult;
use tendermint_rpc::endpoint::block_results::Response as TendermintBlockResultResponse;

use crate::id::Id;
use crate::transaction::TransactionExitStatus;

#[derive(Debug, Clone)]
pub enum EventKind {
    Applied,
    SendPacket,
    Unknown,
}

impl From<&String> for EventKind {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "tx/applied" => Self::Applied,
            "send_packet" => Self::SendPacket,
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
    pub attributes: Option<TxAttributesType>,
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
            batch_results: value.0.iter().fold(
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
            batch_errors: value.0.iter().fold(
                BTreeMap::default(),
                |mut acc, (tx_hash, result)| {
                    let tx_id = Id::from(*tx_hash);
                    let result = if let Ok(result) = result {
                        result
                            .vps_result
                            .errors
                            .iter()
                            .map(|(address, error)| {
                                (Id::from(address.clone()), error.clone())
                            })
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

#[derive(Debug, Clone)]
pub struct TxApplied {
    pub code: TxEventStatusCode,
    pub gas: u64,
    pub hash: Id,
    pub height: u64,
    pub batch: BatchResults,
    pub info: String,
    pub masp_refs: Option<HashMap<u64, Vec<MaspRef>>>,
}

#[derive(Debug, Clone)]
pub enum MaspRef {
    Native(String),
    Ibc(String),
}

#[derive(Debug, Clone, Default)]
pub struct SendPacket {
    pub source_port: String,
    pub dest_port: String,
    pub source_channel: String,
    pub dest_channel: String,
    pub timeout_timestamp: u64,
    pub sequence: String,
}

#[derive(Debug, Clone)]
pub enum TxAttributesType {
    TxApplied(TxApplied),
    SendPacket(SendPacket),
}

impl TxAttributesType {
    pub fn deserialize(
        event_kind: &EventKind,
        attributes: &BTreeMap<String, String>,
    ) -> Option<Self> {
        match event_kind {
            EventKind::Unknown => None,
            EventKind::SendPacket => {
                let source_port =
                    attributes.get("packet_src_port").unwrap().to_owned();
                let dest_port =
                    attributes.get("packet_dst_port").unwrap().to_owned();
                let source_channel =
                    attributes.get("packet_src_channel").unwrap().to_owned();
                let dest_channel =
                    attributes.get("packet_dst_channel").unwrap().to_owned();
                let sequence =
                    attributes.get("packet_sequence").unwrap().to_owned();
                let timeout_timestamp = attributes
                    .get("packet_timeout_timestamp")
                    .unwrap_or(&"0".to_string())
                    .parse::<u64>()
                    .unwrap_or_default()
                    .to_owned();

                Some(Self::SendPacket(SendPacket {
                    source_port,
                    dest_port,
                    source_channel,
                    dest_channel,
                    timeout_timestamp,
                    sequence,
                }))
            }
            EventKind::Applied => Some(Self::TxApplied(TxApplied {
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
                masp_refs: attributes
                    .get("masp_data_refs")
                    .map(|data| {
                        if let Ok(data) =
                            serde_json::from_str::<IndexedMaspData>(data)
                        {
                            let refs = data
                                .masp_refs
                                .0
                                .iter()
                                .map(|masp_ref| match masp_ref {
                                    MaspTxRef::MaspSection(masp_tx_id) => {
                                        MaspRef::Native(masp_tx_id.to_string())
                                    }
                                    MaspTxRef::IbcData(hash) => {
                                        MaspRef::Ibc(hash.to_string())
                                    }
                                })
                                .collect();
                            Some(HashMap::from_iter([(
                                data.tx_index.0 as u64,
                                refs,
                            )]))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default(),
                batch: attributes
                    .get("batch")
                    .map(|batch_result| {
                        let tx_result: TxResult<String> =
                            serde_json::from_str(batch_result).unwrap();
                        BatchResults::from(tx_result)
                    })
                    .unwrap(),
                info: attributes.get("info").unwrap().to_owned(),
            })),
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
                    TxAttributesType::deserialize(&kind, &raw_attributes);
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
                    TxAttributesType::deserialize(&kind, &raw_attributes);
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
            .filter_map(|event| {
                if let Some(TxAttributesType::TxApplied(data)) =
                    &event.attributes
                {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .find(|attributes| attributes.hash.eq(tx_hash))
            .map(|attributes| attributes.clone().code)
            .map(TransactionExitStatus::from);

        exit_status.unwrap_or(TransactionExitStatus::Rejected)
    }

    pub fn gas_used(&self, tx_hash: &Id) -> Option<String> {
        self.end_events
            .iter()
            .filter_map(|event| {
                if let Some(TxAttributesType::TxApplied(data)) =
                    &event.attributes
                {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .find(|attributes| attributes.hash.eq(tx_hash))
            .map(|attributes| attributes.gas.to_string())
    }

    pub fn is_inner_tx_accepted(
        &self,
        wrapper_hash: &Id,
        inner_hash: &Id,
    ) -> TransactionExitStatus {
        let exit_status = self
            .end_events
            .iter()
            .filter_map(|event| {
                if let Some(TxAttributesType::TxApplied(data)) =
                    &event.attributes
                {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .find(|attributes| attributes.hash.eq(wrapper_hash))
            .map(|attributes| attributes.batch.is_successful(inner_hash))
            .map(|successful| match successful {
                true => TransactionExitStatus::Applied,
                false => TransactionExitStatus::Rejected,
            });
        exit_status.unwrap_or(TransactionExitStatus::Rejected)
    }
}
