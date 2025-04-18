use serde::Deserialize;
use validator_derive::Validate;

use crate::shared::role::Role;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserDto {
  #[serde(rename = "userName")]
  pub user_name: String,
  pub role: Role,
}
