use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use axum_extra::extract::Query;

use crate::dto::crawler_state::CrawlerStateQueryParams;
use crate::error::api::ApiError;
use crate::response::crawler_state::CrawlersTimestamps;
use crate::state::common::CommonState;

pub async fn get_crawlers_timestamps(
    _headers: HeaderMap,
    Query(query): Query<CrawlerStateQueryParams>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<CrawlersTimestamps>>, ApiError> {
    let crawler_names = query.crawler_names.unwrap_or(vec![]);

    let timestamps = state
        .crawler_state_service
        .get_timestamps(crawler_names)
        .await?;

    Ok(Json(timestamps))
}
