use serde::{Deserialize, Serialize};

use crate::shared::role::Role;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserRto {
  pub uuid: String,
  #[serde(rename = "userName")]
  pub user_name: String,
  pub role: Role,
}
