use namada_sdk::ibc::core::{channel::types::msgs::PacketMsg, handler::types::msgs::MsgEnvelope};
use shared::{block_result::{BlockResult, SendPacket, TxAttributesType}, transaction::{InnerTransaction, TransactionKind}};

pub fn get_ibc_packets(block_results: &BlockResult) -> Vec<SendPacket> {
    block_results
        .end_events
        .iter()
        .filter_map(|event| {
            if let Some(attributes) = &event.attributes {
                match attributes {
                    TxAttributesType::SendPacket(packet) => {
                        Some(packet.to_owned())
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_ibc_recv_ack(inner_txs: &Vec<InnerTransaction>) {
    inner_txs.iter().filter_map(|tx| {
        match tx.kind {
            TransactionKind::IbcMsgTransfer(ibc_message) => {
                match ibc_message {
                    Some(ibc_message) => match ibc_message.0 {
                        namada_sdk::ibc::IbcMessage::Envelope(msg_envelope) => {
                            match *msg_envelope {
                                MsgEnvelope::Packet(packet_msg) => match packet_msg {
                                    PacketMsg::Recv(msg) => {
                                        None
                                    },
                                    PacketMsg::Ack(msg) => {
                                        None
                                    },
                                    PacketMsg::Timeout(msg) => {
                                        None
                                    },
                                    PacketMsg::TimeoutOnClose(msg) => {
                                        None
                                    },
                                },
                                _ => None
                            }
                        }
                        _ => None
                    },
                    None => None,
                }
            },
            _ => None
        }
    });
}