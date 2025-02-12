use namada_ibc::core::handler::types::msgs::MsgEnvelope;
use namada_ibc::IbcMessage;
use namada_sdk::address::Address;
use namada_sdk::token::Transfer;
use namada_tx::either::IntoEither;

use crate::id::Id;
use crate::ser::{self, TransferData};
use crate::token::Token;
use crate::transaction::TransactionKind;

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

pub fn transfer_to_tx_kind(
    data: Transfer,
    masp_address: &Address,
) -> TransactionKind {
    let has_shielded_section = data.shielded_section_hash.is_some();
    if has_shielded_section && data.sources.is_empty() && data.targets.is_empty() {
        // For fully shielded transaction we don't explicitly write the masp address in the sources nor targets
        return TransactionKind::ShieldedTransfer(Some(data.into()));
    }

    let (all_sources_are_masp, any_sources_are_masp) = data
        .sources
        .iter()
        .fold((true, false), |(all, any), (acc, _)| {
            let is_masp = acc.owner.eq(masp_address);
            (all && is_masp, any || is_masp)
        });

    let (all_targets_are_masp, any_targets_are_masp) = data
        .targets
        .iter()
        .fold((true, false), |(all, any), (acc, _)| {
            let is_masp = acc.owner.eq(masp_address);
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
    masp_address: &Address,
    ibc_data: IbcMessage<Transfer>,
) -> TransactionKind {
    match ibc_data {
        namada_ibc::IbcMessage::Envelope(msg_envelope) => {
            // FIXME: improve here
            // Look for a possible shilding transaction in the envelope
            //FIXME: why don't we use the transaction data? Cause we reextract it later, should we just cache it here?
            if let Some(shielding) =
                namada_ibc::extract_masp_tx_from_envelope(&envelope)
            {
                //FIXME: ok can I construct data from here? Maybe from the packet
                //FIXME: I could justs populate it with source IBC and target masp, but I need the token and the amount
                //FIXME: no we also need the actual original sender in the other chain
                TransactionKind::IbcShieldingTransfer((Some(ibc_data), data))
            } else {
                TransactionKind::IbcMsgTransfer(Some(IbcMessage(ibc_data)))
            }
        }
        namada_ibc::IbcMessage::Transfer(transfer) => {
            match transfer.transfer {
                Some(shielded_transfer) => shielded_transfer_to_ibc_tx_kind(shielded_transfer, masp_address, ibc_data),
                None => {
                    // Transparent transfer, extract data from the ibc packet
                    //FIXME: todo
                }
            }
        }
        namada_ibc::IbcMessage::NftTransfer(nft_transfer) => {
            match nft_transfer.transfer {
                Some(shielded_nft_transfer) => shielded_transfer_to_ibc_tx_kind(shielded_nft_transfer, masp_address, ibc_data)
                None => {
                    // Transparent transfer, extract data from the ibc packet
                    //FIXME: todo
                }
            }
        }
    }
}

// FIXME: clean this up
fn get_transfer_data_from_ibc_packet(ibc_msg: IbcMessage<Transfer>) -> TransferData {
    let mut transfer = TransferData::default();
        
    match ibc_msg {
        namada_ibc::IbcMessage::Envelope(msg_envelope) => {
        }
        namada_ibc::IbcMessage::Transfer(transfer) => {
            let a = transfer.message.packet_data;
            //FIXME: do we also need the data from other chains, i.e. senders/receivers not on namada?
            //FIXME: all wrong here, get the entry and increment
            transfer.sources.insert(a.sender, a.token);
            transfer.targets.insert(a.receiver, a.token); 
            //FIXME: also need to look at the possible shielding/unshieldign
            
        }
        namada_ibc::IbcMessage::NftTransfer(nft_transfer) => {
    }}

    transfer
}

fn shielded_transfer_to_ibc_tx_kind(
    data: Transfer,
    masp_address: &Address,
    ibc_data: IbcMessage<Transfer>,
) -> TransactionKind {
    let (all_sources_are_masp, any_sources_are_masp) = data
        .sources
        .iter()
        .fold((true, false), |(all, any), (acc, _)| {
            let is_masp = acc.owner.eq(masp_address);
            (all && is_masp, any || is_masp)
        });

    let (all_targets_are_masp, any_targets_are_masp) = data
        .targets
        .iter()
        .fold((true, false), |(all, any), (acc, _)| {
            let is_masp = acc.owner.eq(masp_address);
            (all && is_masp, any || is_masp)
        });

    //FIXME: does this make sense for ibc?
    //FIXME: so, first of all, dependign on the packet we already know if it is unshielding or shielding. Also in this function it  cannot be transparent. Also I believe it can't possibly be mixed for ibc cause the packet ultimately supports a single sender and single receiver
    //FIXME: actually I think transfer is not even used for shielded ibc txs, only its shielded field. The rest of the fields are only used if masp fee payment
    //FIXME: ah so this is a problem, I need to merge the shielded data, I need both the fee target and the actual ibc target on the other chain
    match (
        all_sources_are_masp,
        any_sources_are_masp,
        all_targets_are_masp,
        any_targets_are_masp,
    ) {
        (true, _, _, false) => TransactionKind::IbcUnshieldingTransfer((
            Some(ser::IbcMessage(ibc_data)),
            data.into(),
        )),
        (false, _, true, _) => TransactionKind::IbcShieldingTransfer((
            Some(ser::IbcMessage(ibc_data)),
            data.into(),
        )),
        //FIXME: this can't happen
        (false, _, false, _) => TransactionKind::IbcTrasparentTransfer((
            Some(ser::IbcMessage(ibc_data)),
            data.into(),
        )),
        //FIXME: this can't happen
        _ => TransactionKind::MixedTransfer(Some(data.into())),
    }
}
