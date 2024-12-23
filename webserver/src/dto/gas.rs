use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::gas::GasError;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct GasEstimateQuery {
    #[validate(range(min = 1, max = 100))]
    pub bond: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub claim_rewards: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub unbond: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub transparent_transfer: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub shielded_transfer: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub shielding_transfer: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub unshielding_transfer: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub vote: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub ibc: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub withdraw: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub reveal_pk: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub redelegate: Option<u64>,
    #[validate(range(min = 1, max = 20))]
    pub signatures: Option<u64>,
    #[validate(range(min = 1, max = 100000))]
    pub tx_size: Option<u64>,
}

impl GasEstimateQuery {
    pub fn is_valid(&self) -> Result<(), GasError> {
        let res = [
            self.bond,
            self.claim_rewards,
            self.unbond,
            self.transparent_transfer,
            self.shielded_transfer,
            self.shielding_transfer,
            self.unshielding_transfer,
            self.vote,
            self.withdraw,
            self.ibc,
            self.reveal_pk,
            self.redelegate,
        ]
        .iter()
        .any(|field| field.is_some());

        if res {
            Ok(())
        } else {
            Err(GasError::InvalidQueryParams)
        }
    }
}
