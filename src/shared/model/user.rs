use crate::shared::role::Role;

#[derive(Clone, PartialEq, Eq)]
pub struct User {
  pub uuid: String,
  pub user_name: String,
  pub role: Role
}
