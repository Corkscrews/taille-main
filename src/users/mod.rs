pub mod dto;
pub mod model;
pub mod repository;
pub mod rto;

use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use dto::create_user_dto::CreateUserDto;
use dto::get_user_dto::GetUserDto;
use model::access_token_claims::AccessTokenClaims;
use nanoid::nanoid;
use repository::user_repository::UserRepositoryError;
use rto::get_user_rto::GetUserRto;
use validator::Validate;

use crate::custom_nanoid;
use crate::shared::http_error::HttpError;
use crate::shared::role::Role;
use crate::shared::rto::created_rto::CreatedRto;
use crate::users::model::user::User;
use crate::users::repository::user_repository::{CreateUser, UserRepository};

pub async fn get_user<UR: UserRepository>(
  user_repository: web::Data<UR>,
  path: web::Path<GetUserDto>,
  auth: AccessTokenClaims,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = path.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  user_repository
    .find_one(&path.uuid)
    .await
    .filter(|user| auth.is_user_allowed(user))
    .ok_or_else(user_not_found)
    .map(user_found)
    .unwrap_or_else(|err| err)
}

fn user_found(user: User) -> HttpResponse {
  HttpResponse::Ok()
    .content_type("application/json")
    .append_header((header::LOCATION, format!("/v1/users/{}", user.uuid)))
    .json(GetUserRto::from(user))
}

fn user_not_found() -> HttpResponse {
  HttpResponse::NotFound()
    .content_type("application/json")
    .json(HttpError::from("User not found"))
}

// Transform User domain to RTO
impl From<User> for GetUserRto {
  fn from(user: User) -> Self {
    Self {
      uuid: user.uuid,
      user_name: user.user_name,
      role: user.role,
    }
  }
}

pub async fn create_user<UR: UserRepository>(
  user_repository: web::Data<UR>,
  dto: web::Json<CreateUserDto>,
  auth: AccessTokenClaims,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = dto.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }
  if auth.role != Role::Admin && auth.role != Role::Manager {
    return HttpResponse::Forbidden().body("Forbidden");
  }
  user_repository
    .create(CreateUser::from(dto.into_inner()))
    .await
    .map(user_created)
    .unwrap_or_else(failed_create_user)
}

fn user_created(user: User) -> HttpResponse {
  HttpResponse::Created()
    .content_type("application/json")
    .append_header((header::LOCATION, format!("/v1/users/{}", user.uuid)))
    .json(CreatedRto::from(user))
}

fn failed_create_user(error: UserRepositoryError) -> HttpResponse {
  HttpResponse::InternalServerError().finish()
}

impl From<CreateUserDto> for CreateUser {
  fn from(dto: CreateUserDto) -> Self {
    Self {
      uuid: custom_nanoid(),
      user_name: dto.user_name,
      role: dto.role,
    }
  }
}

// Transform User domain to RTO
impl From<User> for CreatedRto {
  fn from(user: User) -> Self {
    Self { uuid: user.uuid }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, RwLock};

  use actix_web::{http::StatusCode, HttpRequest};
  use chrono::Utc;
  use nanoid::nanoid;
  use repository::user_repository::tests::InMemoryUserRepository;

  use crate::helpers::tests::{
    create_fake_access_token_claims, http_request, parse_http_response,
  };

  use super::*;

  #[actix_web::test]
  async fn test_get_user_successful() {
    let jwt_secret = custom_nanoid();
    let uuid = custom_nanoid();

    let user = User {
      uuid: uuid.clone(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      user_name: "John Doe".to_string(),
      role: Role::Admin,
    };

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = get_user(
      web::Data::from(Arc::new(InMemoryUserRepository {
        users: RwLock::new(vec![user]),
      })),
      web::Path::from(GetUserDto { uuid: uuid.clone() }),
      create_fake_access_token_claims(),
    )
    .await;

    let rto: GetUserRto =
      parse_http_response(responder, &request, StatusCode::OK).await;

    // Assertions
    assert_eq!(rto.user_name, "John Doe");
    assert_eq!(rto.role, Role::Admin);
    assert_eq!(rto.uuid, uuid);
  }

  #[actix_web::test]
  async fn test_get_user_uuid_not_found() {
    let jwt_secret = custom_nanoid();
    let uuid = custom_nanoid();

    let user = User {
      uuid: uuid.clone(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      user_name: "John Doe".to_string(),
      role: Role::Admin,
    };

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = get_user(
      web::Data::from(Arc::new(InMemoryUserRepository {
        users: RwLock::new(vec![user]),
      })),
      web::Path::from(GetUserDto { uuid: custom_nanoid() }),
      create_fake_access_token_claims(),
    )
    .await;

    let rto: HttpError =
      parse_http_response(responder, &request, StatusCode::NOT_FOUND).await;

    // Assertions
    assert_eq!(rto.message, "User not found");
  }

  #[test]
  fn test_create_user_dto_to_create_user() {
    let dto = CreateUserDto {
      user_name: "test_user".to_string(),
      role: Role::Admin,
    };

    let user: CreateUser = dto.clone().into();

    assert_eq!(user.user_name, dto.user_name);
    assert_eq!(user.role, dto.role);
    assert!(!user.uuid.is_empty()); // Ensure UUID is generated
  }

  #[test]
  fn test_user_to_get_user_rto() {
    let user = User {
      uuid: "test_uuid".to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      user_name: "test_user".to_string(),
      role: Role::Admin,
    };

    let rto: GetUserRto = user.clone().into();

    assert_eq!(rto.uuid, user.uuid);
    assert_eq!(rto.user_name, user.user_name);
    assert_eq!(rto.role, user.role);
  }

  #[test]
  fn test_user_to_create_user_rto() {
    let user = User {
      uuid: "test_uuid".to_string(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      user_name: "test_user".to_string(),
      role: Role::Admin,
    };
    let rto: CreatedRto = user.clone().into();
    assert_eq!(rto.uuid, user.uuid);
  }
}
