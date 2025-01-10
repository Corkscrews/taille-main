use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::{postgres::PgRow, Pool, Postgres};
use thiserror::Error;

use crate::trips::model::Trip;
use crate::shared::database::Database;

#[derive(Debug, Error)]
pub enum TripRepositoryError {
  #[error("Database error: {0}")]
  DatabaseError(#[from] sqlx::Error),

  #[error("Serialization error: {0}")]
  SerializationError(#[from] serde_json::Error),

  #[error("Other error: {0}")]
  Other(String),
}

pub trait TripRepository {
  async fn find_one(&self, uuid: String) -> Option<Trip>;
  async fn create(&self, create_trip: CreateTrip)
    -> Result<Trip, TripRepositoryError>;
}

pub struct TripRepositoryImpl {
  pool: Arc<Pool<Postgres>>,
}

impl TripRepositoryImpl {
  pub fn new(database: Arc<Database>) -> Self {
    Self {
      pool: database.pool.clone(),
    }
  }
}

impl TripRepository for TripRepositoryImpl {
  async fn find_one(&self, uuid: String) -> Option<Trip> {
    let rows = sqlx::query("SELECT * FROM trips WHERE uuid = ? LIMIT 1")
      .bind(uuid)
      .map(|row: PgRow| Trip::from(row))
      .fetch_one(&*self.pool)
      .await;
    rows.ok()
  }

  async fn create(
    &self,
    create_trip: CreateTrip,
  ) -> Result<Trip, TripRepositoryError> {
    let query = r#"
      INSERT INTO users (uuid, start_coords, end_coords)
      VALUES ($1, $2, $3)
      RETURNING uuid, start_coords, end_coords
    "#;
    sqlx::query(query)
      .bind(&create_trip.uuid)
      .bind(&create_trip.start_coords)
      .bind(&create_trip.end_coords)
      .map(|row: PgRow| Trip::from(row))
      .fetch_one(&*self.pool)
      .await
      .map_err(TripRepositoryError::from)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTrip {
  pub uuid: String,
  pub start_coords: String,
  pub end_coords: String,
}

impl From<PgRow> for Trip {
  fn from(row: PgRow) -> Self {
    Self {
      uuid: row.get("uuid"),
      created_at: row.get::<DateTime<Utc>, _>("created_at"),
      updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
      start_coords: row.get("start_coords"),
      end_coords: row.get("end_coords")
    }
  }
}

#[cfg(test)]
pub mod tests {
  use chrono::Utc;
  use crate::trips::model::Trip;
  use std::sync::RwLock;

  use super::{CreateTrip, TripRepository, TripRepositoryError};

  pub struct InMemoryTripRepository {
    pub trips: RwLock<Vec<Trip>>,
  }

  impl InMemoryTripRepository {
    pub fn new() -> Self {
      Self {
        trips: RwLock::new(Vec::new()),
      }
    }
  }

  impl TripRepository for InMemoryTripRepository {
    async fn find_one(&self, uuid: String) -> Option<Trip> {
      let trips = self.trips.read().unwrap(); // Acquire read lock
      trips.iter().find(|trip| trip.uuid == uuid).cloned()
    }

    async fn create(
      &self,
      create_trip: CreateTrip,
    ) -> Result<Trip, TripRepositoryError> {
      let mut trips = self.trips.write().unwrap(); // Acquire write lock
      let trip = Trip {
        uuid: create_trip.uuid,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        start_coords: String::from(""),
        end_coords: String::from(""),
      };
      trips.push(trip.clone());
      Ok(trip)
    }
  }
}