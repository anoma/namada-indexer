use namada_sdk::ibc::core::channel::types::acknowledgement::AcknowledgementStatus;
use namada_sdk::ibc::core::channel::types::msgs::PacketMsg;
use namada_sdk::ibc::core::handler::types::msgs::MsgEnvelope;
use shared::block_result::{BlockResult, TxAttributesType};
use shared::gas::GasEstimation;
use shared::transaction::{
    IbcAck, IbcAckStatus, IbcSequence, InnerTransaction, TransactionKind,
    WrapperTransaction,
};

pub fn get_ibc_packets(
    block_results: &BlockResult,
    inner_txs: &[InnerTransaction],
) -> Vec<IbcSequence> {
    let mut ibc_txs = inner_txs
        .iter()
        .filter_map(|tx| {
            if tx.is_ibc() && tx.was_successful() {
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

pub fn get_gas_estimates(
    inner_txs: &[InnerTransaction],
    wrapper_txs: &[WrapperTransaction],
) -> Vec<GasEstimation> {
    wrapper_txs
        .iter()
        .map(|wrapper_tx| {
            let mut gas_estimate = GasEstimation::new(wrapper_tx.tx_id.clone());
            gas_estimate.signatures = wrapper_tx.total_signatures;
            gas_estimate.size = wrapper_tx.size;

            inner_txs
                .iter()
                .filter(|inner_tx| {
                    inner_tx.was_successful()
                        && inner_tx.wrapper_id.eq(&wrapper_tx.tx_id)
                })
                .for_each(|tx| match tx.kind {
                    TransactionKind::TransparentTransfer(_)
                    | TransactionKind::MixedTransfer(_) => {
                        gas_estimate.increase_mixed_transfer()
                    }
                    TransactionKind::IbcMsgTransfer(_) => {
                        gas_estimate.increase_ibc_msg_transfer()
                    }
                    TransactionKind::Bond(_) => gas_estimate.increase_bond(),
                    TransactionKind::Redelegation(_) => {
                        gas_estimate.increase_redelegation()
                    }
                    TransactionKind::Unbond(_) => {
                        gas_estimate.increase_unbond()
                    }
                    TransactionKind::Withdraw(_) => {
                        gas_estimate.increase_withdraw()
                    }
                    TransactionKind::ClaimRewards(_) => {
                        gas_estimate.increase_claim_rewards()
                    }
                    TransactionKind::ProposalVote(_) => {
                        gas_estimate.increase_vote()
                    }
                    TransactionKind::RevealPk(_) => {
                        gas_estimate.increase_reveal_pk()
                    }
                    TransactionKind::ShieldedTransfer(_) => {
                        gas_estimate.increase_shielded_transfer()
                    }
                    TransactionKind::ShieldingTransfer(_) => {
                        gas_estimate.increase_shielding_transfer()
                    }
                    TransactionKind::UnshieldingTransfer(_) => {
                        gas_estimate.increase_ibc_unshielding_transfer()
                    }
                    TransactionKind::IbcShieldingTransfer(_) => {
                        gas_estimate.increase_ibc_shielding_transfer()
                    }
                    _ => (),
                });
            gas_estimate
        })
        .collect()
}
