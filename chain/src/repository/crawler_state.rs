use anyhow::Context;
use chrono::NaiveDateTime;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::crawler_state::{ChainStateInsertDb, CrawlerNameDb};
use orm::schema::crawler_state;
use shared::crawler_state::{ChainCrawlerState, CrawlerName};

pub fn upsert_crawler_state(
    transaction_conn: &mut PgConnection,
    crawler_state: ChainCrawlerState,
) -> anyhow::Result<()> {
    diesel::insert_into(crawler_state::table)
        .values::<&ChainStateInsertDb>(
            &(CrawlerName::Chain, crawler_state).into(),
        )
        .on_conflict(crawler_state::name)
        .do_update()
        .set((
            crawler_state::timestamp.eq(excluded(crawler_state::timestamp)),
            crawler_state::last_processed_block
                .eq(excluded(crawler_state::last_processed_block)),
            crawler_state::last_processed_epoch
                .eq(excluded(crawler_state::last_processed_epoch)),
            crawler_state::first_block_in_epoch
                .eq(excluded(crawler_state::first_block_in_epoch)),
        ))
        .execute(transaction_conn)
        .context("Failed to update crawler state in db")?;

    anyhow::Ok(())
}

pub fn update_crawler_timestamp(
    transaction_conn: &mut PgConnection,
    timestamp: NaiveDateTime,
) -> anyhow::Result<()> {
    diesel::update(crawler_state::table)
        .filter(crawler_state::name.eq(CrawlerNameDb::from(CrawlerName::Chain)))
        .set(crawler_state::timestamp.eq(timestamp))
        .execute(transaction_conn)
        .context("Failed to update crawler timestamp in db")?;

    anyhow::Ok(())
}

#[cfg(test)]
mod tests {

    use anyhow::Context;
    use diesel::QueryDsl;
    use orm::crawler_state::ChainCrawlerStateDb;
    use shared::block::{BlockHeight, Epoch};
    use test_helpers::db::TestDb;

    use super::*;

    // Test case for successfully inserting a new crawler state
    #[tokio::test]
    async fn test_upsert_crawler_state_insert_success() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let crawler_state = ChainCrawlerState {
                last_processed_block: 1,
                last_processed_epoch: 1,
                first_block_in_epoch: 1,
                timestamp: 1,
            };
            upsert_crawler_state(conn, crawler_state.clone())?;

            let queried_state = query_chain_state(conn)?;

            assert_eq!(queried_state, crawler_state);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    // Test case for successfully updating an existing crawler state
    #[tokio::test]
    async fn test_upsert_crawler_state_update_success() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let initial_crawler_state = ChainCrawlerState {
                last_processed_block: 1,
                last_processed_epoch: 1,
                first_block_in_epoch: 1,
                timestamp: 1,
            };

            let crawler_state = ChainCrawlerState {
                last_processed_block: 2,
                last_processed_epoch: 2,
                first_block_in_epoch: 2,
                timestamp: 2,
            };
            seed_chain_state(conn, initial_crawler_state.clone())?;

            upsert_crawler_state(conn, crawler_state.clone())?;

            let queried_state = query_chain_state(conn)?;

            assert_eq!(queried_state, crawler_state);

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    // Test case for successfully updating an existing crawler timestamp
    #[tokio::test]
    async fn test_update_crawler_timestamp_success() {
        let db = TestDb::new();

        db.run_test(|conn| {
            let initial_crawler_state = ChainCrawlerState {
                last_processed_block: 1,
                last_processed_epoch: 1,
                first_block_in_epoch: 1,
                timestamp: 1,
            };

            seed_chain_state(conn, initial_crawler_state.clone())?;

            let new_timestamp = chrono::Utc::now().naive_utc();
            update_crawler_timestamp(conn, new_timestamp)?;

            let queried_state = query_chain_state(conn)?;

            assert_eq!(
                queried_state,
                ChainCrawlerState {
                    timestamp: new_timestamp.and_utc().timestamp(),
                    ..initial_crawler_state
                }
            );

            anyhow::Ok(())
        })
        .await
        .expect("Failed to run test");
    }

    fn seed_chain_state(
        conn: &mut PgConnection,
        crawler_state: ChainCrawlerState,
    ) -> anyhow::Result<()> {
        let crawler_state_db = ChainStateInsertDb::from((
            CrawlerName::Chain,
            crawler_state.clone(),
        ));
        diesel::insert_into(crawler_state::table)
            .values::<ChainStateInsertDb>(crawler_state_db)
            .execute(conn)
            .context("Failed to update crawler_state in db")?;

        anyhow::Ok(())
    }

    fn query_chain_state(
        conn: &mut PgConnection,
    ) -> anyhow::Result<ChainCrawlerState> {
        let crawler_state: ChainCrawlerStateDb = crawler_state::table
            .filter(crawler_state::name.eq(CrawlerNameDb::Chain))
            .select((
                crawler_state::dsl::last_processed_block,
                crawler_state::dsl::last_processed_epoch,
                crawler_state::dsl::first_block_in_epoch,
                crawler_state::dsl::timestamp,
            ))
            .first(conn)
            .context("Failed to query all balances")?;

        Ok(ChainCrawlerState {
            last_processed_block: crawler_state.last_processed_block
                as BlockHeight,
            last_processed_epoch: crawler_state.last_processed_epoch as Epoch,
            first_block_in_epoch: crawler_state.first_block_in_epoch
                as BlockHeight,
            timestamp: crawler_state.timestamp.and_utc().timestamp(),
        })
    }
}
