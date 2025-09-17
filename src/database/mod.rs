pub mod models;
pub mod schema;

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::config::DatabaseConfig;
use std::time::Duration;

pub async fn init_pool(config: &DatabaseConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Some(Duration::from_secs(config.idle_timeout)))
        .connect(&config.url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}