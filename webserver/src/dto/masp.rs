use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MaspAggregatesQueryParams {
    pub token: Option<String>,
}
