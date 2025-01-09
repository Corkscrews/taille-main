use serde::Deserialize;
use validator_derive::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTripDto {
  pub start_coords: String,
  pub end_coords: String,
}
