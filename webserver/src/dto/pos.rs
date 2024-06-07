use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub page: Option<u64>,
    pub state: Option<Vec<ValidatorStateDto>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MyValidatorKindDto {
    WithBonds,
    WithUnbonds,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct MyValidatorQueryParams {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u64>,

    #[validate(length(
        min = 1,
        message = "Address query parameter cannot be empty"
    ))]
    pub addresses: Vec<String>,

    pub kind: MyValidatorKindDto,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct BondsDto {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct UnbondsDto {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct WithdrawsDto {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u64>,
}
