use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use namada_sdk::events::extend::{IndexedMaspData, MaspTxRefs};
use namada_tx::data::TxResult;
use tendermint_rpc::endpoint::block_results::Response as TendermintBlockResultResponse;

use crate::id::Id;
use crate::transaction::TransactionExitStatus;

#[derive(Debug, Clone)]
pub enum EventKind {
    Applied,
    SendPacket,
    FungibleTokenPacket,
    Unknown,
}

impl From<&String> for EventKind {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "tx/applied" => Self::Applied,
            "send_packet" => Self::SendPacket,
            "fungible_token_packet" => Self::FungibleTokenPacket,
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
            "0" => Self::Ok,
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
    fn is_successful(&self, tx_id: &Id) -> bool {
        self.batch_results.get(tx_id).map_or(false, |res| *res)
    }
}

#[derive(Clone)]
pub struct TxApplied {
    pub code: TxEventStatusCode,
    pub gas: u64,
    pub hash: Id,
    pub height: u64,
    pub batch: BatchResults,
    pub info: String,
    pub masp_refs: HashMap<u64, MaspTxRefs>,
}

impl fmt::Debug for TxApplied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            code,
            gas,
            hash,
            height,
            batch,
            info,
            masp_refs,
        } = self;

        f.debug_struct("TxApplied")
            .field("code", code)
            .field("gas", gas)
            .field("hash", hash)
            .field("height", height)
            .field("batch", batch)
            .field("info", info)
            .field("masp_refs_len", &masp_refs.len())
            .finish()
    }
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

#[derive(Debug, Clone, Default)]
pub struct FungibleTokenPacket {
    pub sender: String,
    pub receiver: String,
    pub denom: String,
    pub memo: String,
    pub amount: BigDecimal,
}

#[derive(Debug, Clone)]
pub enum TxAttributesType {
    TxApplied(TxApplied),
    SendPacket(SendPacket),
    FungibleTokenPacket {
        is_ack: bool,
        success: bool,
        packet: FungibleTokenPacket,
    },
}

impl TxAttributesType {
    pub fn deserialize(
        event_kind: &EventKind,
        attributes: &BTreeMap<String, String>,
    ) -> Option<Self> {
        match event_kind {
            EventKind::Unknown => None,
            EventKind::FungibleTokenPacket => {
                let (is_ack, success) =
                    if let Some(success) = attributes.get("success") {
                        (false, success == "\u{1}" || success == "true")
                    } else {
                        (
                            true,
                            attributes
                                .get("acknowledgement")
                                .map(|ack| ack.starts_with("result:"))
                                .unwrap_or_default(),
                        )
                    };

                let sender = attributes.get("sender")?.to_owned();
                let receiver = attributes.get("receiver")?.to_owned();
                let denom = attributes.get("denom")?.to_owned();
                let memo = attributes.get("memo")?.to_owned();
                let amount =
                    attributes.get("amount")?.parse::<BigDecimal>().ok()?;

                Some(TxAttributesType::FungibleTokenPacket {
                    is_ack,
                    success,
                    packet: FungibleTokenPacket {
                        sender,
                        receiver,
                        denom,
                        memo,
                        amount,
                    },
                })
            }
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
                            let refs = data.masp_refs.0.to_vec();
                            HashMap::from_iter([(
                                data.tx_index.0 as u64,
                                MaspTxRefs(refs),
                            )])
                        } else {
                            HashMap::default()
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

    pub fn masp_refs(&self, wrapper_hash: &Id, index: u64) -> MaspTxRefs {
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
            .find(|attributes| attributes.hash.eq(wrapper_hash))
            .map(|event| {
                event.masp_refs.get(&index).cloned().unwrap_or_default()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAttribute {
        key: String,
        value: String,
    }

    struct TestEvent {
        kind: String,
        attributes: Vec<TestAttribute>,
    }

    #[test]
    fn ibc_fungible_token_events() {
        let mut events: Vec<_> = example_events()
            .into_iter()
            .filter_map(|test_event| {
                let kind = EventKind::from(&test_event.kind);
                let attributes = test_event
                    .attributes
                    .into_iter()
                    .map(|TestAttribute { key, value }| (key, value))
                    .collect();

                let parsed_attributes =
                    TxAttributesType::deserialize(&kind, &attributes);

                parsed_attributes.as_ref()?;

                Some(Event {
                    kind,
                    attributes: parsed_attributes,
                })
            })
            .collect();

        assert_eq!(events.len(), 1);
        assert!(matches!(
            events.remove(0),
            Event {
                kind: EventKind::FungibleTokenPacket,
                attributes: Some(
                    TxAttributesType::FungibleTokenPacket {
                        is_ack: true,
                        success: true,
                        packet: FungibleTokenPacket {
                            sender,
                            receiver,
                            denom,
                            memo,
                            amount,
                        },
                    },
                ),
            }
            if
                sender == "osmo1m8wg4vxkefhs374qxmmqpyusgz289wmulex5qdwpfx7jnrxzer5s9cv83q"
                    && receiver == "mantra1drlcrf9drkhwayf37dephlxc0uqg5zcqjd7er8"
                    && denom == "transfer/channel-85077/uom"
                    && memo.is_empty()
                    && amount == "19582".parse().unwrap()
        ));
    }

    fn example_events() -> Vec<TestEvent> {
        vec![
            TestEvent {
                kind: "acknowledge_packet".to_owned(),
                attributes: vec![
                    TestAttribute {
                        key: "packet_timeout_height".to_owned(),
                        value: "0-0".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_timeout_timestamp".to_owned(),
                        value: "1739790400335658800".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_sequence".to_owned(),
                        value: "56446".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_src_port".to_owned(),
                        value: "transfer".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_src_channel".to_owned(),
                        value: "channel-85077".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_dst_port".to_owned(),
                        value: "transfer".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_dst_channel".to_owned(),
                        value: "channel-0".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_channel_ordering".to_owned(),
                        value: "ORDER_UNORDERED".to_owned(),
                    },
                    TestAttribute {
                        key: "packet_connection".to_owned(),
                        value: "connection-2766".to_owned(),
                    },
                    TestAttribute {
                        key: "connection_id".to_owned(),
                        value: "connection-2766".to_owned(),
                    },
                    TestAttribute {
                        key: "msg_index".to_owned(),
                        value: "1".to_owned(),
                    },
                ],
            },
            TestEvent {
                kind: "fungible_token_packet".to_owned(),
                attributes: vec![
                    TestAttribute {
                        key: "module".to_owned(),
                        value: "transfer".to_owned(),
                    },
                    TestAttribute {
                        key: "sender".to_owned(),
                        value: "osmo1m8wg4vxkefhs374qxmmqpyusgz289wmulex5qdwpfx7jnrxzer5s9cv83q".to_owned(),
                    },
                    TestAttribute {
                        key: "receiver".to_owned(),
                        value: "mantra1drlcrf9drkhwayf37dephlxc0uqg5zcqjd7er8".to_owned(),
                    },
                    TestAttribute {
                        key: "denom".to_owned(),
                        value: "transfer/channel-85077/uom".to_owned(),
                    },
                    TestAttribute {
                        key: "amount".to_owned(),
                        value: "19582".to_owned(),
                    },
                    TestAttribute {
                        key: "memo".to_owned(),
                        value: "".to_owned(),
                    },
                    TestAttribute {
                        key: "acknowledgement".to_owned(),
                        value: "result:\"\\001\" ".to_owned(),
                    },
                    TestAttribute {
                        key: "msg_index".to_owned(),
                        value: "1".to_owned(),
                    },
                ],
            },
            TestEvent {
                kind: "fungible_token_packet".to_owned(),
                attributes: vec![
                    TestAttribute {
                        key: "success".to_owned(),
                        value: "\u{1}".to_owned(),
                    },
                    TestAttribute {
                        key: "msg_index".to_owned(),
                        value: "1".to_owned(),
                    },
                ],
            },
            TestEvent {
                kind: "ibc_transfer".to_owned(),
                attributes: vec![
                    TestAttribute {
                        key: "sender".to_owned(),
                        value: "noble1dljdlrhg8s9kj2yq2u90q4e6kdxll8njpywgzh".to_owned(),
                    },
                    TestAttribute {
                        key: "receiver".to_owned(),
                        value: "mantra1dljdlrhg8s9kj2yq2u90q4e6kdxll8njzv3yer".to_owned(),
                    },
                    TestAttribute {
                        key: "amount".to_owned(),
                        value: "4995952207".to_owned(),
                    },
                    TestAttribute {
                        key: "denom".to_owned(),
                        value: "uusdc".to_owned(),
                    },
                    TestAttribute {
                        key: "memo".to_owned(),
                        value: "".to_owned(),
                    },
                ],
            },
        ]
    }
}
