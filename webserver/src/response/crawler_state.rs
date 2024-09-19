use serde::{Deserialize, Serialize};
use shared::crawler_state::CrawlerTimestamp as SharedCrawlerTimestamp;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlerTimestamp {
    pub name: String,
    pub timestamp: i64,
}

impl CrawlerTimestamp {
    pub fn empty(name: String) -> Self {
        Self { name, timestamp: 0 }
    }
}

impl From<&SharedCrawlerTimestamp> for CrawlerTimestamp {
    fn from(shared: &SharedCrawlerTimestamp) -> Self {
        Self {
            name: shared.name.clone(),
            timestamp: shared.timestamp,
        }
    }
}
