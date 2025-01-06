use serde::{Deserialize, Serialize};

use crate::shared::{model::user::User, role::Role};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
  uuid: String,
  pub role: Role,
  exp: usize,
  iat: usize,
}

impl AccessTokenClaims {
  pub fn is_user_allowed(&self, user: &User) -> bool {
    if self.uuid == user.uuid {
      return true;
    }
    if self.role == Role::Admin || self.role == Role::Manager {
      return true;
    }
    // TODO: Handle other cases, role based.
    false
  }
}
