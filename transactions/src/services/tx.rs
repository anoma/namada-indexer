use namada_sdk::ibc::core::{
    channel::types::{acknowledgement::AcknowledgementStatus, msgs::PacketMsg},
    handler::types::msgs::MsgEnvelope,
};
use shared::{
    block_result::{BlockResult, SendPacket, TxAttributesType},
    transaction::{IbcAck, IbcAckStatus, InnerTransaction, TransactionKind},
};

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

pub fn get_ibc_recv_ack(inner_txs: &Vec<InnerTransaction>) -> Vec<IbcAck> {
    inner_txs.iter().filter_map(|tx| match tx.kind.clone() {
        TransactionKind::IbcMsgTransfer(ibc_message) => match ibc_message {
            Some(ibc_message) => match ibc_message.0 {
                namada_sdk::ibc::IbcMessage::Envelope(msg_envelope) => {
                    match *msg_envelope {
                        MsgEnvelope::Packet(packet_msg) => match packet_msg {
                            PacketMsg::Recv(_) => None,
                            PacketMsg::Ack(msg) => {
                                let ack = match serde_json::from_slice::<
                                    AcknowledgementStatus,
                                >(
                                    msg.acknowledgement.as_bytes()
                                ) {
                                    Ok(status) => IbcAck {
                                        sequence_number: msg.packet.seq_on_a.to_string(),
                                        source_port: msg.packet.port_id_on_a.to_string(),
                                        dest_port: msg.packet.port_id_on_b.to_string(),
                                        source_channel: msg.packet.chan_id_on_a.to_string(),
                                        dest_channel: msg.packet.chan_id_on_b.to_string(),
                                        status: match status {
                                            AcknowledgementStatus::Success(_) => IbcAckStatus::Success,
                                            AcknowledgementStatus::Error(_) => IbcAckStatus::Fail,
                                        },
                                    },
                                    Err(_) => IbcAck {
                                        sequence_number: msg.packet.seq_on_a.to_string(),
                                        source_port: msg.packet.port_id_on_a.to_string(),
                                        dest_port: msg.packet.port_id_on_b.to_string(),
                                        source_channel: msg.packet.chan_id_on_a.to_string(),
                                        dest_channel: msg.packet.chan_id_on_b.to_string(),
                                        status: IbcAckStatus::Unknown,
                                    },
                                };
                                Some(ack)
                            }
                            PacketMsg::Timeout(msg) => Some(IbcAck {
                                sequence_number: msg.packet.seq_on_a.to_string(),
                                source_port: msg.packet.port_id_on_a.to_string(),
                                dest_port: msg.packet.port_id_on_b.to_string(),
                                source_channel: msg.packet.chan_id_on_a.to_string(),
                                dest_channel: msg.packet.chan_id_on_b.to_string(),
                                status: IbcAckStatus::Timeout,
                            }),
                            PacketMsg::TimeoutOnClose(msg) => Some(IbcAck {
                                sequence_number: msg.packet.seq_on_a.to_string(),
                                source_port: msg.packet.port_id_on_a.to_string(),
                                dest_port: msg.packet.port_id_on_b.to_string(),
                                source_channel: msg.packet.chan_id_on_a.to_string(),
                                dest_channel: msg.packet.chan_id_on_b.to_string(),
                                status: IbcAckStatus::Timeout,
                            }),
                        },
                        _ => None,
                    }
                }
                _ => None,
            },
            None => None,
        },
        _ => None,
    }).collect()
}
