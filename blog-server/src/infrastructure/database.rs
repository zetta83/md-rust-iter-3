use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, migrate};
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("running database migrations");
    migrate!("./migrations").run(pool).await?;
    info!("migrations completed");
    Ok(())
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await?;
    info!("Created database connection pool");
    Ok(pool)
}
