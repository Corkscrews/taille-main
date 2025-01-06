use std::sync::Arc;

use sqlx::{PgPool, Pool, Postgres};

pub struct Database {
  pub pool: Arc<Pool<Postgres>>,
}

impl Database {
  pub async fn new() -> Self {
    let pool = prepare_pool().await;
    Self { pool: Arc::new(pool) }
  }
}

pub async fn prepare_pool() -> Pool<Postgres> {
  PgPool::connect("postgres://user:password@localhost/database")
    .await
    .unwrap()
}