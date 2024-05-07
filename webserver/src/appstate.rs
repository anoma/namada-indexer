use std::sync::{Arc, Mutex};
use std::{env, process::exit};

use chrono::offset::Utc;
use deadpool_diesel::postgres::{Object, Pool as DbPool};
use deadpool_redis::{Config, Connection, Pool as CachePool};

#[derive(Clone)]
pub struct AppState {
    db: DbPool,
    cache: CachePool,
    next_cache_timestamp: Arc<Mutex<u64>>,
}

impl AppState {
    pub fn new(db_url: String, cache_url: String) -> Self {
        let max_pool_size = env::var("DATABASE_POOL_SIZE")
            .unwrap_or_else(|_| 16.to_string())
            .parse::<usize>()
            .unwrap_or(16_usize);
        let pool_manager = deadpool_diesel::Manager::new(
            db_url,
            deadpool_diesel::Runtime::Tokio1,
        );
        let pool = DbPool::builder(pool_manager)
            .max_size(max_pool_size)
            .build();

        let pool = match pool {
            Ok(pool) => pool,
            Err(e) => {
                tracing::info!(
                    "Error building postgres pool: {}",
                    e.to_string()
                );
                exit(1);
            }
        };

        let cache_pool = Config::from_url(cache_url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1));
        let cache_pool = match cache_pool {
            Ok(pool) => pool,
            Err(e) => {
                tracing::info!("Error building redis pool: {}", e.to_string());
                exit(1);
            }
        };

        Self {
            db: pool,
            cache: cache_pool,
            next_cache_timestamp: Arc::new(Mutex::new(0)),
        }
    }

    pub fn next_cache_timestamp(&self) -> u64 {
        const FIVE_MINUTES_IN_SECONDS: u64 = 5 * 60;

        let current_ts: u64 = Utc::now()
            .naive_utc()
            .and_utc()
            .timestamp()
            .try_into()
            .expect("Failed to convert Unix timestamp from i64 to u64");
        let mut stored_ts = self.next_cache_timestamp.lock().unwrap();

        if current_ts.saturating_sub(*stored_ts) > FIVE_MINUTES_IN_SECONDS {
            *stored_ts = current_ts + FIVE_MINUTES_IN_SECONDS;
        }

        *stored_ts
    }

    pub async fn get_db_connection(&self) -> Object {
        self.db.get().await.unwrap()
    }

    pub async fn get_cache_connection(&self) -> Connection {
        self.cache.get().await.unwrap()
    }
}
