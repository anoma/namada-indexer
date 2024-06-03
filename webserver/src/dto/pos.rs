use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct PoSQueryParams {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u64>,
}
