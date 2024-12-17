use namada_sdk::ibc::core::channel::types::acknowledgement::AcknowledgementStatus;
use namada_sdk::ibc::core::channel::types::msgs::PacketMsg;
use namada_sdk::ibc::core::handler::types::msgs::MsgEnvelope;
use shared::block_result::{BlockResult, TxAttributesType};
use shared::ser::IbcMessage;
use shared::transaction::{
    IbcAck, IbcAckStatus, IbcSequence, InnerTransaction, TransactionExitStatus,
    TransactionKind,
};

pub fn get_ibc_packets(
    block_results: &BlockResult,
    inner_txs: &[InnerTransaction],
) -> Vec<IbcSequence> {
    let mut ibc_txs = inner_txs
        .iter()
        .filter_map(|tx| {
            if matches!(
                tx.kind,
                TransactionKind::IbcMsgTransfer(Some(IbcMessage(_)))
            ) && matches!(tx.exit_code, TransactionExitStatus::Applied)
            {
                Some(tx.tx_id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ibc_txs.reverse();

    block_results
        .end_events
        .iter()
        .filter_map(|event| {
            if let Some(attributes) = &event.attributes {
                match attributes {
                    TxAttributesType::SendPacket(packet) => Some(IbcSequence {
                        sequence_number: packet.sequence.clone(),
                        source_port: packet.source_port.clone(),
                        dest_port: packet.dest_port.clone(),
                        source_channel: packet.source_channel.clone(),
                        dest_channel: packet.dest_channel.clone(),
                        timeout: packet.timeout_timestamp,
                        tx_id: ibc_txs.pop().unwrap(),
                    }),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_ibc_ack_packet(inner_txs: &[InnerTransaction]) -> Vec<IbcAck> {
    inner_txs.iter().filter_map(|tx| match tx.kind.clone() {
        TransactionKind::IbcMsgTransfer(Some(ibc_message)) => match ibc_message.0 {
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
            },
            _ => None
        },
        _ => None,
    }).collect()
}
