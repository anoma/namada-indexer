use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct GasEstimateQuery {
    #[validate(range(min = 1, max = 100))]
    pub page: Option<u64>,
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
}
