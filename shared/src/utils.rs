use namada_ibc::IbcMessage;
use namada_sdk::address::Address;
use namada_sdk::token::Transfer;

use crate::id::Id;
use crate::ser;
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
        (false, _, true, _, true) => {
            TransactionKind::ShieldingTransfer(Some(data.into()))
        }
        (false, _, false, _, false) => {
            TransactionKind::TransparentTransfer(Some(data.into()))
        }
        _ => TransactionKind::MixedTransfer(Some(data.into())),
    }
}

pub fn transfer_to_ibc_tx_kind(
    data: Transfer,
    masp_address: &Address,
    ibc_data: IbcMessage<Transfer>,
) -> TransactionKind {
    let has_shielded_section = data.shielded_section_hash.is_some();

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
        (true, _, _, false, true) => TransactionKind::IbcUnshieldingTransfer((
            Some(ser::IbcMessage(ibc_data)),
            data.into(),
        )),
        (false, _, true, _, true) => TransactionKind::IbcShieldingTransfer((
            Some(ser::IbcMessage(ibc_data)),
            data.into(),
        )),
        (false, _, false, _, false) => TransactionKind::IbcTrasparentTransfer(
            (Some(ser::IbcMessage(ibc_data)), data.into()),
        ),
        _ => TransactionKind::MixedTransfer(Some(data.into())),
    }
}
