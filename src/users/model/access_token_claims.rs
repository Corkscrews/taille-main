use serde::{Deserialize, Serialize};

use crate::shared::role::Role;

use super::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
  pub uuid: String,
  pub role: Role,
  pub exp: usize,
  pub iat: usize,
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
