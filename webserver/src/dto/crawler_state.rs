use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrawlerNameDto {
    Chain,
    Governance,
    Parameters,
    Pos,
    Rewards,
    Transactions,
}

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct CrawlerStateQueryParams {
    pub crawler_names: Option<Vec<CrawlerNameDto>>,
}
