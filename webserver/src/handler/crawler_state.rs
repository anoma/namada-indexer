use axum::{extract::State, http::HeaderMap, Json};
use axum_extra::extract::Query;

use crate::{
    dto::crawler_state::CrawlerStateQueryParams,
    response::crawler_state::CrawlersTimestamps, state::common::CommonState,
};

pub async fn get_crawlers_state(
    _headers: HeaderMap,
    Query(query): Query<CrawlerStateQueryParams>,
    State(state): State<CommonState>,
) -> Json<Vec<CrawlersTimestamps>> {
    let crawler_names = query.crawler_names.unwrap_or(vec![]);

    let crawler_state = state
        .crawler_state_service
        .get_timestamps(crawler_names)
        .await;

    Json(crawler_state)
}
