use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IbcRateLimit {
    pub token_address: Option<String>,
    pub throughput_limit: Option<u64>,
}
