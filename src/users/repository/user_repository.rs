use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::{postgres::PgRow, Pool, Postgres};
use thiserror::Error;

use crate::shared::role::Role;
use crate::{shared::database::Database, users::model::user::User};

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
  async fn find_one(&self, uuid: &str) -> Option<User>;
  async fn create(
    &self,
    create_user: CreateUser,
  ) -> Result<User, UserRepositoryError>;
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
  async fn find_one(&self, uuid: &str) -> Option<User> {
    let rows = sqlx::query("SELECT * FROM users WHERE uuid = ? LIMIT 1")
      .bind(uuid)
      .map(|row: PgRow| User::from(row))
      .fetch_one(&*self.pool)
      .await;
    rows.ok()
  }

  async fn create(
    &self,
    create_user: CreateUser,
  ) -> Result<User, UserRepositoryError> {
    let query = r#"
      INSERT INTO users (uuid, user_name, role)
      VALUES ($1, $2, $3)
      RETURNING uuid, user_name, role
    "#;
    sqlx::query(query)
      .bind(&create_user.uuid)
      .bind(&create_user.user_name)
      .bind(serde_json::to_string(&create_user.role).unwrap())
      .map(|row: PgRow| User::from(row))
      .fetch_one(&*self.pool)
      .await
      .map_err(UserRepositoryError::from)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateUser {
  pub uuid: String,
  pub user_name: String,
  pub role: Role,
}

impl From<PgRow> for User {
  fn from(row: PgRow) -> Self {
    Self {
      uuid: row.get("uuid"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
      user_name: row.get("user_name"),
      role: serde_json::from_str(row.get("role")).unwrap(),
    }
  }
}

#[cfg(test)]
pub mod tests {
  use chrono::Utc;

  use super::{CreateUser, UserRepository, UserRepositoryError};
  use crate::users::model::user::User;
  use std::sync::RwLock;

  pub struct InMemoryUserRepository {
    pub users: RwLock<Vec<User>>,
  }

  impl InMemoryUserRepository {
    pub fn new() -> Self {
      Self {
        users: RwLock::new(Vec::new()),
      }
    }
  }

  impl UserRepository for InMemoryUserRepository {
    async fn find_one(&self, uuid: &str) -> Option<User> {
      let users = self.users.read().unwrap(); // Acquire read lock
      users.iter().find(|user| user.uuid == uuid).cloned()
    }

    async fn create(
      &self,
      user: CreateUser,
    ) -> Result<User, UserRepositoryError> {
      let mut users = self.users.write().unwrap(); // Acquire write lock
      let user = User {
        uuid: user.uuid,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        user_name: user.user_name,
        role: user.role,
      };
      users.push(user.clone());
      Ok(user)
    }
  }
}
