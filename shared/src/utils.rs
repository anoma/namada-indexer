use namada_ibc::apps::nft_transfer::types::PORT_ID_STR as NFT_PORT_ID_STR;
use namada_ibc::apps::transfer::types::{
    packet::PacketData as FtPacketData, PORT_ID_STR as FT_PORT_ID_STR,
};
use namada_ibc::core::handler::types::msgs::MsgEnvelope;
use namada_sdk::address::Address;
use namada_sdk::token::Transfer;

use crate::id::Id;
use crate::ser::{self, TransferData};
use crate::token::Token;
use crate::transaction::TransactionKind;

pub(crate) const MASP_ADDRESS: Address =
    Address::Internal(namada_sdk::address::InternalAddress::Masp);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BalanceChange {
    pub address: Id,
    pub token: Token,
}

impl BalanceChange {
    pub fn new(address: Id, token: Token) -> Self {
        Self { address, token }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct GovernanceProposalShort {
    pub id: u64,
    pub voting_start_epoch: u64,
    pub voting_end_epoch: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DelegationPair {
    pub validator_address: Id,
    pub delegator_address: Id,
}

pub fn transfer_to_tx_kind(data: Transfer) -> TransactionKind {
    let has_shielded_section = data.shielded_section_hash.is_some();
    if has_shielded_section
        && data.sources.is_empty()
        && data.targets.is_empty()
    {
        // For fully shielded transaction we don't explicitly write the masp address in the sources nor targets
        return TransactionKind::ShieldedTransfer(Some(data.into()));
    }

    let (all_sources_are_masp, any_sources_are_masp) = data
        .sources
        .iter()
        .fold((true, false), |(all, any), (acc, _)| {
            let is_masp = acc.owner.eq(&MASP_ADDRESS);
            (all && is_masp, any || is_masp)
        });

    let (all_targets_are_masp, any_targets_are_masp) = data
        .targets
        .iter()
        .fold((true, false), |(all, any), (acc, _)| {
            let is_masp = acc.owner.eq(&MASP_ADDRESS);
            (all && is_masp, any || is_masp)
        });

    match (
        all_sources_are_masp,
        any_sources_are_masp,
        all_targets_are_masp,
        any_targets_are_masp,
        has_shielded_section,
    ) {
        (true, _, true, _, true) => {
            TransactionKind::ShieldedTransfer(Some(data.into()))
        }
        (true, _, _, false, true) => {
            TransactionKind::UnshieldingTransfer(Some(data.into()))
        }
        (_, false, true, _, true) => {
            TransactionKind::ShieldingTransfer(Some(data.into()))
        }
        (_, false, _, false, false) => {
            TransactionKind::TransparentTransfer(Some(data.into()))
        }
        _ => TransactionKind::MixedTransfer(Some(data.into())),
    }
}

pub fn transfer_to_ibc_tx_kind(
    ibc_data: namada_ibc::IbcMessage<Transfer>,
    native_token: Address,
) -> TransactionKind {
    match &ibc_data {
        namada_ibc::IbcMessage::Envelope(msg_envelope) => {
            if let MsgEnvelope::Packet(
                namada_ibc::core::channel::types::msgs::PacketMsg::Recv(msg),
            ) = msg_envelope.as_ref()
            {
                // Extract transfer info from the packet
                let transfer_data = match msg.packet.port_id_on_b.as_str() {
                    FT_PORT_ID_STR => {
                        let packet_data =
                            serde_json::from_slice::<FtPacketData>(
                                &msg.packet.data,
                            )
                            .expect("Could not deserialize IBC fungible token packet");

                        let (token, denominated_amount) = if packet_data
                            .token
                            .denom
                            .to_string()
                            .contains(&native_token.to_string())
                        {
                            (
                                native_token.clone(),
                                namada_sdk::token::DenominatedAmount::native(
                                    namada_sdk::token::Amount::from_str(
                                        &packet_data.token.amount.to_string(),
                                        0,
                                    )
                                    .expect("Failed conversion of IBC amount to Namada one"),
                                ),
                            )
                        } else {
                            let ibc_trace = format!(
                                "{}/{}/{}",
                                msg.packet.port_id_on_b,
                                msg.packet.chan_id_on_b,
                                packet_data.token.denom
                            );
                            (
                                namada_ibc::trace::convert_to_address(
                                    ibc_trace,
                                )
                                .expect(
                                    "Failed to convert IBC trace to address",
                                ),
                                namada_sdk::token::DenominatedAmount::new(
                                    namada_sdk::token::Amount::from_str(
                                        &packet_data.token.amount.to_string(),
                                        0,
                                    )
                                    .expect("Failed conversion of IBC amount to Namada one"),
                                    0.into(),
                                ),
                            )
                        };

                        TransferData {
                            // TODO: support indexing addresses as string to allow for ibc sources
                            sources: crate::ser::AccountsMap(
                                [(
                                    namada_sdk::token::Account {
                                        owner: namada_sdk::address::IBC,
                                        token: token.clone(),
                                    },
                                    denominated_amount,
                                )]
                                .into(),
                            ),
                            targets: crate::ser::AccountsMap(
                                [(
                                    namada_sdk::token::Account {
                                        owner: packet_data.receiver.try_into().expect("Failed to convert IBC signer to address"),
                                        token,
                                    },
                                    denominated_amount,
                                )]
                                .into(),
                            ),
                            shielded_section_hash: None,
                        }
                    }
                    NFT_PORT_ID_STR => {
                        // TODO: add support for indexing nfts
                        todo!("IBC NFTs are not yet supported for indexing purposes")
                    }
                    _ => {
                        tracing::warn!("Found unsupported IBC packet data");
                        return TransactionKind::IbcMsg(Some(ser::IbcMessage(
                            ibc_data,
                        )));
                    }
                };

                let is_shielding =
                    namada_sdk::ibc::extract_masp_tx_from_envelope(
                        &msg_envelope,
                    )
                    .is_some();
                if is_shielding {
                    TransactionKind::IbcShieldingTransfer((
                        ser::IbcMessage(ibc_data),
                        transfer_data,
                    ))
                } else {
                    TransactionKind::IbcTrasparentTransfer((
                        ser::IbcMessage(ibc_data),
                        transfer_data,
                    ))
                }
            } else {
                TransactionKind::IbcMsg(Some(ser::IbcMessage(ibc_data)))
            }
        }
        namada_ibc::IbcMessage::Transfer(transfer) => {
            let (token, denominated_amount) = if transfer
                .message
                .packet_data
                .token
                .denom
                .to_string()
                .contains(&native_token.to_string())
            {
                (
                    native_token.clone(),
                    namada_sdk::token::DenominatedAmount::native(
                        namada_sdk::token::Amount::from_str(
                            &transfer
                                .message
                                .packet_data
                                .token
                                .amount
                                .to_string(),
                            0,
                        )
                        .expect(
                            "Failed conversion of IBC amount to Namada one",
                        ),
                    ),
                )
            } else {
                let ibc_trace = format!(
                    "{}/{}/{}",
                    transfer.message.port_id_on_a,
                    transfer.message.chan_id_on_a,
                    transfer.message.packet_data.token.denom
                );
                (
                    namada_ibc::trace::convert_to_address(ibc_trace)
                        .expect("Failed to convert IBC trace to address"),
                    namada_sdk::token::DenominatedAmount::new(
                        namada_sdk::token::Amount::from_str(
                            &transfer
                                .message
                                .packet_data
                                .token
                                .amount
                                .to_string(),
                            0,
                        )
                        .expect(
                            "Failed conversion of IBC amount to Namada one",
                        ),
                        0.into(),
                    ),
                )
            };

            let transfer_data = TransferData {
                sources: crate::ser::AccountsMap(
                    [(
                        namada_sdk::token::Account {
                            owner: transfer
                                .message
                                .packet_data
                                .sender
                                .to_owned()
                                .try_into()
                                .expect(
                                    "Failed to convert IBC signer to address",
                                ),
                            token: token.clone(),
                        },
                        denominated_amount,
                    )]
                    .into(),
                ),
                targets: crate::ser::AccountsMap(
                    [(
                        namada_sdk::token::Account {
                            // TODO: support indexing addresses as string to allow for ibc targets
                            owner: namada_sdk::address::IBC,
                            token,
                        },
                        denominated_amount,
                    )]
                    .into(),
                ),
                shielded_section_hash: None,
            };

            if transfer.transfer.is_some() {
                TransactionKind::IbcUnshieldingTransfer((
                    ser::IbcMessage(ibc_data),
                    transfer_data,
                ))
            } else {
                TransactionKind::IbcTrasparentTransfer((
                    ser::IbcMessage(ibc_data),
                    transfer_data,
                ))
            }
        }
        namada_ibc::IbcMessage::NftTransfer(_nft_transfer) => {
            // TODO: add support for indexing nfts
            todo!("IBC NFTs are not yet supported for indexing purposes")
        }
    }
}
