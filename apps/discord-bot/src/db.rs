use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

/// Initialize the PostgreSQL connection pool and run migrations.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database initialized and migrations applied");

    Ok(pool)
}
