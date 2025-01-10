pub mod dto;
pub mod model;
pub mod rto;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use dto::get_trip_dto::GetTripDto;
use jsonwebtoken::{decode, DecodingKey, Validation};
use nanoid::nanoid;
use validator::Validate;

use crate::shared::repository::user_repository::{CreateUser, UserRepository};
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::AppState;

pub async fn get_trip<UR: UserRepository + 'static>(
  data: web::Data<AppState<UR>>,
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

pub async fn create_trip<UR: UserRepository + 'static>(
  data: web::Data<AppState<UR>>,
  dto: web::Json<CreateUserDto>,
  request: HttpRequest,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  HttpResponse::Ok().content_type("application/json").finish()
}
