use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Role {
  #[serde(rename = "admin")]
  Admin,
  #[serde(rename = "manager")]
  Manager,
  #[serde(rename = "driver")]
  Driver,
  #[serde(rename = "customer")]
  Customer,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_serialization() {
    let admin = Role::Admin;
    let serialized =
      serde_json::to_string(&admin).expect("Failed to serialize");
    assert_eq!(serialized, "\"admin\"");

    let manager = Role::Manager;
    let serialized =
      serde_json::to_string(&manager).expect("Failed to serialize");
    assert_eq!(serialized, "\"manager\"");

    let driver = Role::Driver;
    let serialized =
      serde_json::to_string(&driver).expect("Failed to serialize");
    assert_eq!(serialized, "\"driver\"");

    let customer = Role::Customer;
    let serialized =
      serde_json::to_string(&customer).expect("Failed to serialize");
    assert_eq!(serialized, "\"customer\"");
  }

  #[test]
  fn test_deserialization() {
    let admin: Role =
      serde_json::from_str("\"admin\"").expect("Failed to deserialize");
    assert_eq!(admin, Role::Admin);

    let manager: Role =
      serde_json::from_str("\"manager\"").expect("Failed to deserialize");
    assert_eq!(manager, Role::Manager);

    let driver: Role =
      serde_json::from_str("\"driver\"").expect("Failed to deserialize");
    assert_eq!(driver, Role::Driver);

    let customer: Role =
      serde_json::from_str("\"customer\"").expect("Failed to deserialize");
    assert_eq!(customer, Role::Customer);
  }

  #[test]
  fn test_invalid_deserialization() {
    let invalid: Result<Role, _> = serde_json::from_str("\"invalid\"");
    assert!(invalid.is_err());
  }
}
