use chrono::{DateTime, Utc};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trip {
  pub uuid: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub start_coords: String,
  pub end_coords: String,
}
