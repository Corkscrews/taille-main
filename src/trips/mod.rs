pub mod dto;
pub mod model;
pub mod repository;
pub mod rto;

use actix_web::{http::header, web, HttpRequest, HttpResponse, Responder};
use dto::create_trip_dto::CreateTripDto;
use dto::get_trip_dto::GetTripDto;
use jsonwebtoken::{decode, DecodingKey, Validation};
use model::Trip;
use nanoid::nanoid;
use repository::trip_repository::{CreateTrip, TripRepository};
use validator::Validate;

use crate::shared::{config::Config, rto::created_rto::CreatedRto};

pub async fn get_trip<TR: TripRepository>(
  config: web::Data<Config>,
  user_repository: web::Data<TR>,
  path: web::Path<GetTripDto>,
  request: HttpRequest,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = path.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  HttpResponse::Ok().content_type("application/json").finish()
}

pub async fn create_trip<TR: TripRepository>(
  config: web::Data<Config>,
  trip_repository: web::Data<TR>,
  dto: web::Json<CreateTripDto>,
  request: HttpRequest,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  trip_repository
    .create(CreateTrip::from(dto.into_inner()))
    .await
    .map(|trip| {
      HttpResponse::Created()
        .content_type("application/json")
        .append_header((header::LOCATION, format!("/v1/trips/{}", trip.uuid)))
        .json(CreatedRto::from(trip))
    })
    .unwrap_or_else(|error| {
      println!("{}", error);
      HttpResponse::InternalServerError().finish()
    })
}

impl From<CreateTripDto> for CreateTrip {
  fn from(dto: CreateTripDto) -> Self {
    Self {
      uuid: nanoid!(),
      start_coords: dto.start_coords,
      end_coords: dto.end_coords,
    }
  }
}

// Transform User domain to RTO
impl From<Trip> for CreatedRto {
  fn from(trip: Trip) -> Self {
    Self { uuid: trip.uuid }
  }
}
