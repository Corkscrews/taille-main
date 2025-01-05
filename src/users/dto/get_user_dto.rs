use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GetUserDTO {
  pub uuid: String,
}
