use serde::{Deserialize, Serialize};
use validator::Validate;

use super::utils::Pagination;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct PoSQueryParams {
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
}