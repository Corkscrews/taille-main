use std::sync::Arc;

use sqlx::Row;
use sqlx::{postgres::PgRow, Pool, Postgres};
use thiserror::Error;

use crate::{shared::database::Database, shared::model::user::User};

#[derive(Debug, Error)]
pub enum UserRepositoryError {
  #[error("Database error: {0}")]
  DatabaseError(#[from] sqlx::Error),

  #[error("Serialization error: {0}")]
  SerializationError(#[from] serde_json::Error),

  #[error("Other error: {0}")]
  Other(String),
}

pub trait UserRepository {
  async fn find_one(&self, uuid: String) -> Option<User>;
  async fn create(&self, user: User) -> Result<User, UserRepositoryError>;
}

pub struct UserRepositoryImpl {
  pool: Arc<Pool<Postgres>>,
}

impl UserRepositoryImpl {
  pub fn new(database: Arc<Database>) -> Self {
    Self {
      pool: database.pool.clone(),
    }
  }
}

impl UserRepository for UserRepositoryImpl {
  async fn find_one(&self, uuid: String) -> Option<User> {
    let rows = sqlx::query("SELECT * FROM users WHERE uuid = ? LIMIT 1")
      .bind(uuid)
      .map(|row: PgRow| User::from(row))
      .fetch_one(&*self.pool)
      .await;
    rows.ok()
  }

  async fn create(&self, user: User) -> Result<User, UserRepositoryError> {
    let query = r#"
      INSERT INTO users (uuid, user_name, role)
      VALUES ($1, $2, $3)
    "#;
    sqlx::query(query)
      .bind(&user.uuid)
      .bind(&user.user_name)
      .bind(serde_json::to_string(&user.role).unwrap())
      .execute(&*self.pool)
      .await?;
    Ok(user)
  }
}

impl From<PgRow> for User {
  fn from(row: PgRow) -> Self {
    User {
      uuid: row.get("uuid"),
      user_name: row.get("user_name"),
      role: serde_json::from_str(row.get("role")).unwrap(),
    }
  }
}

#[cfg(test)]
pub mod tests {
  use super::{UserRepository, UserRepositoryError};
  use crate::shared::model::user::User;
  use std::sync::RwLock;

  pub struct UserRepositoryMock {
    users: RwLock<Vec<User>>,
  }

  impl UserRepositoryMock {
    pub fn new() -> Self {
      Self {
        users: RwLock::new(Vec::new()),
      }
    }
  }

  impl UserRepository for UserRepositoryMock {
    async fn find_one(&self, uuid: String) -> Option<User> {
      let users = self.users.read().unwrap(); // Acquire read lock
      users.iter().find(|user| user.uuid == uuid).cloned()
    }

    async fn create(&self, user: User) -> Result<User, UserRepositoryError> {
      let mut users = self.users.write().unwrap(); // Acquire write lock
      users.push(user.clone());
      Ok(user)
    }
  }
}
