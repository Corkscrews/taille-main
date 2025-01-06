use serde::Deserialize;
use validator_derive::Validate;

use crate::shared::role::Role;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDTO {
  #[serde(rename = "userName")]
  pub user_name: String,
  pub role: Role,
}
