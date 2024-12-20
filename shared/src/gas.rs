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
    pub unshielding_transfer: u64,
    pub ibc_msg_transfer: u64,
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
            ibc_msg_transfer: 0,
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
}
