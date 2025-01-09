use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GetUserDto {
  pub uuid: String,
}
