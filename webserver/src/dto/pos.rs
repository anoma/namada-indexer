use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize)]
pub enum ValidatorStateDto {
    Consensus,
    BelowCapacity,
    BelowThreshold,
    Inactive,
    Jailed,
    Unknown,
}

impl ValidatorStateDto {
    pub fn all() -> Vec<Self> {
        [
            Self::Consensus,
            Self::BelowCapacity,
            Self::BelowThreshold,
            Self::Inactive,
            Self::Jailed,
            Self::Unknown,
        ]
        .to_vec()
    }
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct PoSQueryParams {
    #[validate(range(min = 1, max = 10000))]
    pub state: Option<Vec<ValidatorStateDto>>,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct MyValidatorQueryParams {
    pub page: Option<u64>,
    #[validate(length(
        min = 1,
        message = "Address query parameter cannot be empty"
    ))]
    pub addresses: Vec<String>,
}
