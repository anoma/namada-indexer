use crate::dto::crawler_state::CrawlerNameDto;
use crate::{appstate::AppState, response::crawler_state::CrawlersTimestamps};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use orm::crawler_state::{CrawlerNameDb, CrawlerStateDb};
use orm::schema::crawler_state;

#[derive(Clone)]
pub struct CrawlerStateService {
    pub(crate) app_state: AppState,
}

impl CrawlerStateService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn get_timestamps(
        &self,
        names: Vec<CrawlerNameDto>,
    ) -> Vec<CrawlersTimestamps> {
        let conn = self.app_state.get_db_connection().await;
        let names_db = names
            .iter()
            .map(Self::to_crawler_name_db)
            .collect::<Vec<_>>();

        let crawlers_db: Vec<CrawlerStateDb> = conn
            .interact(move |conn| {
                let mut query = crawler_state::table.into_boxed();

                if !names_db.is_empty() {
                    query = query.filter(crawler_state::name.eq_any(names_db));
                }

                query
                    .select(crawler_state::all_columns)
                    .get_results(conn)
                    // TODO:
                    .unwrap_or(vec![])
            })
            .await
            // TODO:
            .unwrap_or(vec![]);

        let crawlers: Vec<CrawlersTimestamps> = crawlers_db
            .into_iter()
            .map(|crawler| CrawlersTimestamps {
                name: crawler.name.to_string(),
                timestamp: crawler.timestamp.and_utc().timestamp(),
            })
            .collect();

        crawlers
    }

    fn to_crawler_name_db(value: &CrawlerNameDto) -> CrawlerNameDb {
        match value {
            CrawlerNameDto::Chain => CrawlerNameDb::Chain,
            CrawlerNameDto::Governance => CrawlerNameDb::Governance,
            CrawlerNameDto::Parameters => CrawlerNameDb::Parameters,
            CrawlerNameDto::Pos => CrawlerNameDb::Pos,
            CrawlerNameDto::Rewards => CrawlerNameDb::Rewards,
            CrawlerNameDto::Transactions => CrawlerNameDb::Transactions,
        }
    }
}
