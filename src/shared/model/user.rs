use crate::shared::role::Role;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
  pub uuid: String,
  pub user_name: String,
  pub role: Role,
}
