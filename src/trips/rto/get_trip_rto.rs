use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTripRto {
  pub uuid: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "driverUuid")]
  pub driver_uuid: Option<String>,
  #[serde(rename = "consumerUuid")]
  pub consumer_uuid: String
}
