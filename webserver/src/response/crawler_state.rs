use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlersTimestamps {
    pub name: String,
    pub timestamp: i64,
}
