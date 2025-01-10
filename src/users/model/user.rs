use chrono::{DateTime, Utc};

use crate::shared::role::Role;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
  pub uuid: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub user_name: String,
  pub role: Role,
}
