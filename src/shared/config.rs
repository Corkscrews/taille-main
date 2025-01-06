use std::env;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
  pub master_key: String,
  pub jwt_secret: String,
}

impl Default for Config {
  fn default() -> Self {
    let master_key =
      env::var("MASTER_KEY").unwrap_or_else(|_| "DEV_MASTER_KEY".to_string());
    let jwt_secret =
      env::var("JWT_SECRET").unwrap_or_else(|_| "DEV_JWT_SECRET".to_string());
    Self {
      master_key,
      jwt_secret,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_config_runner() {
    test_default_config();
    test_default_config_with_missing_env_vars();
  }

  fn test_default_config() {
    // Temporarily set environment variables
    env::set_var("MASTER_KEY", "TEST_MASTER_KEY");
    env::set_var("JWT_SECRET", "TEST_JWT_SECRET");

    let config = Config::default();
    assert_eq!(config.master_key, "TEST_MASTER_KEY");
    assert_eq!(config.jwt_secret, "TEST_JWT_SECRET");

    // Clean up environment variables
    env::remove_var("MASTER_KEY");
    env::remove_var("JWT_SECRET");
  }

  fn test_default_config_with_missing_env_vars() {
    // Ensure environment variables are unset
    env::remove_var("MASTER_KEY");
    env::remove_var("JWT_SECRET");

    let config = Config::default();
    assert_eq!(config.master_key, "DEV_MASTER_KEY");
    assert_eq!(config.jwt_secret, "DEV_JWT_SECRET");
  }

  #[test]
  fn test_serialization() {
    let config = Config {
      master_key: "key123".to_string(),
      jwt_secret: "secret123".to_string(),
    };

    let serialized =
      serde_json::to_string(&config).expect("Failed to serialize");
    assert!(serialized.contains("key123"));
    assert!(serialized.contains("secret123"));
  }

  #[test]
  fn test_deserialization() {
    let json = r#"{
      "master_key": "key123",
      "jwt_secret": "secret123"
    }"#;

    let config: Config =
      serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(config.master_key, "key123");
    assert_eq!(config.jwt_secret, "secret123");
  }
}
