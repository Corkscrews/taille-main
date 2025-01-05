use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GetUserRTO {
  pub uuid: String,
  #[serde(rename = "userName")]
  pub user_name: String,
}
