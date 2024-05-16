use std::env;
use std::process::exit;

use deadpool_diesel::postgres::{Object, Pool as DbPool};
use deadpool_redis::{Config, Connection, Pool as CachePool};

#[derive(Clone)]
pub struct AppState {
    db: DbPool,
    cache: CachePool,
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
        }
    }

    pub async fn get_db_connection(&self) -> Object {
        self.db.get().await.unwrap()
    }

    pub async fn get_cache_connection(&self) -> Connection {
        self.cache.get().await.unwrap()
    }
}
