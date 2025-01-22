use crate::balance::Amount;
use crate::id::Id;

#[derive(Clone, Debug)]
pub struct GasPrice {
    pub token: String,
    pub amount: Amount,
}

#[derive(Clone, Debug)]
pub struct GasEstimation {
    pub wrapper_id: Id,
    pub transparent_transfer: u64,
    pub shielded_transfer: u64,
    pub shielding_transfer: u64,
    pub ibc_unshielding_transfer: u64,
    pub ibc_shielding_transfer: u64,
    pub unshielding_transfer: u64,
    pub ibc_msg_transfer: u64,
    pub mixed_transfer: u64,
    pub bond: u64,
    pub redelegation: u64,
    pub unbond: u64,
    pub withdraw: u64,
    pub claim_rewards: u64,
    pub vote_proposal: u64,
    pub reveal_pk: u64,
    pub size: u64,
    pub signatures: u64,
}

impl GasEstimation {
    pub fn new(tx_id: Id) -> Self {
        Self {
            wrapper_id: tx_id,
            transparent_transfer: 0,
            shielded_transfer: 0,
            shielding_transfer: 0,
            unshielding_transfer: 0,
            ibc_shielding_transfer: 0,
            ibc_unshielding_transfer: 0,
            ibc_msg_transfer: 0,
            mixed_transfer: 0,
            bond: 0,
            redelegation: 0,
            unbond: 0,
            withdraw: 0,
            claim_rewards: 0,
            vote_proposal: 0,
            reveal_pk: 0,
            size: 0,
            signatures: 0,
        }
    }

    pub fn increase_transparent_transfer(&mut self) {
        self.transparent_transfer += 1
    }

    pub fn increase_shielded_transfer(&mut self) {
        self.shielded_transfer += 1
    }

    pub fn increase_shielding_transfer(&mut self) {
        self.shielding_transfer += 1
    }

    pub fn increase_unshielding_transfer(&mut self) {
        self.unshielding_transfer += 1
    }

    pub fn increase_mixed_transfer(&mut self) {
        self.mixed_transfer += 1
    }

    pub fn increase_ibc_shielding_transfer(&mut self) {
        self.ibc_shielding_transfer += 1
    }

    pub fn increase_ibc_unshielding_transfer(&mut self) {
        self.ibc_unshielding_transfer += 1
    }

    pub fn increase_ibc_msg_transfer(&mut self) {
        self.ibc_msg_transfer += 1
    }

    pub fn increase_bond(&mut self) {
        self.bond += 1
    }

    pub fn increase_redelegation(&mut self) {
        self.redelegation += 1
    }

    pub fn increase_unbond(&mut self) {
        self.unbond += 1
    }

    pub fn increase_withdraw(&mut self) {
        self.withdraw += 1
    }

    pub fn increase_claim_rewards(&mut self) {
        self.claim_rewards += 1
    }

    pub fn increase_vote(&mut self) {
        self.vote_proposal += 1
    }

    pub fn increase_reveal_pk(&mut self) {
        self.reveal_pk += 1
    }
}
