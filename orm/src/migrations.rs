use deadpool_diesel::postgres::Object;
use deadpool_diesel::InteractError;
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn run_migrations(conn: &Object) -> Result<(), InteractError> {
    conn.interact(|transaction_conn| {
        transaction_conn
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run all migrations");
    })
    .await
    .map(|_| ())
}
