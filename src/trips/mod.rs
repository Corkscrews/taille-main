pub mod dto;
pub mod model;
pub mod repository;
pub mod rto;

use actix_web::{http::header, web, HttpResponse, Responder};
use dto::create_trip_dto::CreateTripDto;
use dto::get_trip_dto::GetTripDto;
use model::Trip;
use nanoid::nanoid;
use repository::trip_repository::{CreateTrip, TripRepository, TripRepositoryError};
use rto::get_trip_rto::GetTripRto;
use validator::Validate;

use crate::{custom_nanoid, shared::{http_error::HttpError, rto::created_rto::CreatedRto}, users::model::access_token_claims::AccessTokenClaims};

pub async fn get_trip<TR: TripRepository>(
  trip_repository: web::Data<TR>,
  path: web::Path<GetTripDto>,
  auth: AccessTokenClaims,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = path.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  trip_repository
    .find_one(&path.uuid)
    .await
    .filter(|trip| trip.consumer_uuid == auth.uuid || trip.driver_uuid == Some(auth.uuid))
    .ok_or_else(trip_not_found)
    .map(trip_found)
    .unwrap_or_else(|err| err)
}

fn trip_found(trip: Trip) -> HttpResponse {
  HttpResponse::Ok()
    .content_type("application/json")
    .append_header((header::LOCATION, format!("/v1/trips/{}", trip.uuid)))
    .json(GetTripRto::from(trip))
}

fn trip_not_found() -> HttpResponse {
  HttpResponse::NotFound()
    .content_type("application/json")
    .json(HttpError::from("Trip not found"))
}

// Transform User domain to RTO
impl From<Trip> for GetTripRto {
  fn from(trip: Trip) -> Self {
    Self {
      uuid: trip.uuid,
      driver_uuid: trip.driver_uuid,
      consumer_uuid: trip.consumer_uuid
    }
  }
}

pub async fn create_trip<TR: TripRepository>(
  trip_repository: web::Data<TR>,
  dto: web::Json<CreateTripDto>,
  auth: AccessTokenClaims,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  trip_repository
    .create(CreateTrip::from(auth, dto.into_inner()))
    .await
    .map(trip_created)
    .unwrap_or_else(failed_create_trip)
}

fn trip_created(trip: Trip) -> HttpResponse {
  HttpResponse::Created()
    .content_type("application/json")
    .append_header((header::LOCATION, format!("/v1/trips/{}", trip.uuid)))
    .json(CreatedRto::from(trip))
}

fn failed_create_trip(error: TripRepositoryError) -> HttpResponse {
  HttpResponse::InternalServerError().finish()
}

impl CreateTrip {
  fn from(auth: AccessTokenClaims, dto: CreateTripDto) -> Self {
    Self {
      uuid: custom_nanoid(),
      start_coords: dto.start_coords,
      end_coords: dto.end_coords,
      driver_uuid: None,
      consumer_uuid: auth.uuid
    }
  }
}

// Transform User domain to RTO
impl From<Trip> for CreatedRto {
  fn from(trip: Trip) -> Self {
    Self { uuid: trip.uuid }
  }
}
