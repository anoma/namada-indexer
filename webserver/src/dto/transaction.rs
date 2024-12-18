use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHistoryQueryParams {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u64>,
    #[validate(length(min = 1, max = 10))]
    pub addresses: Vec<String>,
}
