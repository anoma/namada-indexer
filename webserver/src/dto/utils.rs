use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct Pagination {
    #[validate(range(min = 1, max = 10000))]
    pub page: u64,
}
