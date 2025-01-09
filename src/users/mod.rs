pub mod dto;
pub mod model;
pub mod rto;

use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use dto::create_user_dto::CreateUserDto;
use dto::get_user_dto::GetUserDto;
use jsonwebtoken::{decode, DecodingKey, Validation};
use model::access_token_claims::AccessTokenClaims;
use nanoid::nanoid;
use rto::get_user_rto::GetUserRto;
use validator::Validate;

use crate::shared::config::Config;
use crate::shared::http_error::HttpError;
use crate::shared::model::user::User;
use crate::shared::repository::user_repository::UserRepository;
use crate::shared::role::Role;
use crate::shared::rto::created_rto::CreatedRto;
use crate::AppState;

pub async fn get_user<UR: UserRepository + 'static>(
  data: web::Data<AppState<UR>>,
  path: web::Path<GetUserDto>,
  request: HttpRequest,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = path.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  // User from the JWT. Needed to verify if the user has permission to access the user
  // from the query below.
  let auth = find_auth_user(request, &data.config).await;
  if auth.is_none() {
    return invalid_auth_header();
  }
  let auth = auth.unwrap();

  // TODO: This solution below is vulnerable to time based attacks, transform the login
  // process into a time constant solution to prevent those issues.
  // Call `find_one` with `await` on the repository instance
  let user = data.user_repository.find_one(path.uuid.clone()).await;

  if user.is_none() {
    return user_not_found();
  }
  let user = user.unwrap();

  if !auth.is_user_allowed(&user) {
    return user_not_found();
  }

  HttpResponse::Ok()
    .content_type("application/json")
    .json(GetUserRto::from(user))
}

pub async fn create_user<UR: UserRepository + 'static>(
  data: web::Data<AppState<UR>>,
  dto: web::Json<CreateUserDto>,
  request: HttpRequest,
) -> impl Responder {
  // User from the JWT. Needed to verify if the user has permission to access the user
  // from the query below.
  let auth = find_auth_user(request, &data.config).await;
  if auth.is_none() {
    return invalid_auth_header();
  }
  let auth = auth.unwrap();
  if auth.role != Role::Admin && auth.role != Role::Manager {
    return HttpResponse::Forbidden().body("Forbidden");
  }

  data
    .user_repository
    .create(User::from(dto.into_inner()))
    .await
    .map(|user| {
      HttpResponse::Created()
        .content_type("application/json")
        .append_header((header::LOCATION, format!("/v1/users/{}", user.uuid)))
        .json(CreatedRto::from(user))
    })
    .unwrap_or_else(|error| {
      println!("{}", error);
      HttpResponse::InternalServerError().finish()
    })
}

// TODO: shouldn't this be an middleware?
fn invalid_auth_header() -> HttpResponse {
  HttpResponse::BadRequest()
    .content_type("application/json")
    .json(HttpError::from("Invalid Authorization header"))
}

fn user_not_found() -> HttpResponse {
  HttpResponse::NotFound()
    .content_type("application/json")
    .json(HttpError::from("User not found"))
}

async fn find_auth_user(
  request: HttpRequest,
  config: &Config,
) -> Option<AccessTokenClaims> {
  // Extract the Authorization header
  let authorization_header = match request.headers().get("Authorization") {
    Some(header_value) => match header_value.to_str() {
      Ok(value) => value,
      Err(_) => return None,
    },
    None => return None,
  };
  let token = authorization_header.replace("Bearer ", "");

  let decode_result = decode::<AccessTokenClaims>(
    &token,
    &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
    &Validation::default(),
  );

  if decode_result.is_err() {
    return None;
  }
  let decode_result = decode_result.unwrap();

  Some(decode_result.claims)
}

impl From<CreateUserDto> for User {
  fn from(dto: CreateUserDto) -> Self {
    Self {
      uuid: nanoid!(),
      user_name: dto.user_name,
      role: dto.role,
    }
  }
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

// Transform User domain to RTO
impl From<User> for CreatedRto {
  fn from(user: User) -> Self {
    Self { uuid: user.uuid }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::RwLock;

  use actix_web::http::StatusCode;
  use nanoid::nanoid;

  use crate::{helpers::tests::{http_request, parse_http_response}, shared::{
    config::Config, repository::user_repository::tests::InMemoryUserRepository,
  }};

  use super::*;

  #[actix_web::test]
  async fn test_get_user_successful() {
    let jwt_secret = nanoid!();
    let uuid = nanoid!();

    let user = User {
      uuid: uuid.clone(),
      user_name: "John Doe".to_string(),
      role: Role::Admin,
    };

    let app_state = AppState {
      user_repository: InMemoryUserRepository { 
        users: RwLock::new(vec![user])
       },
      config: Config {
        master_key: nanoid!(),
        jwt_secret: jwt_secret.clone(),
      },
    };

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = get_user(
      web::Data::new(app_state), 
      web::Path::from(GetUserDto { uuid: uuid.clone() }), 
      request.clone()
    ).await;

    let rto: GetUserRto = parse_http_response(
      responder, 
      &request, 
      StatusCode::OK
    ).await;

    // Assertions
    assert_eq!(rto.user_name, "John Doe");
    assert_eq!(rto.role, Role::Admin);
    assert_eq!(rto.uuid, uuid);
  }

  #[actix_web::test]
  async fn test_get_user_uuid_not_found() {
    let jwt_secret = nanoid!();
    let uuid = nanoid!();

    let user = User {
      uuid: uuid.clone(),
      user_name: "John Doe".to_string(),
      role: Role::Admin,
    };

    let app_state = AppState {
      user_repository: InMemoryUserRepository { 
        users: RwLock::new(vec![user])
       },
      config: Config {
        master_key: nanoid!(),
        jwt_secret: jwt_secret.clone(),
      },
    };

    let request: HttpRequest = http_request(&jwt_secret);

    let responder = get_user(
      web::Data::new(app_state), 
      web::Path::from(GetUserDto { uuid: nanoid!() }), 
      request.clone()
    ).await;

    let rto: HttpError = parse_http_response(
      responder, 
      &request, 
      StatusCode::NOT_FOUND
    ).await;

    // Assertions
    assert_eq!(rto.message, "User not found");
  }

  #[test]
  fn test_create_user_dto_to_user() {
    let dto = CreateUserDto {
      user_name: "test_user".to_string(),
      role: Role::Admin,
    };

    let user: User = dto.clone().into();

    assert_eq!(user.user_name, dto.user_name);
    assert_eq!(user.role, dto.role);
    assert!(!user.uuid.is_empty()); // Ensure UUID is generated
  }

  #[test]
  fn test_user_to_get_user_rto() {
    let user = User {
      uuid: "test_uuid".to_string(),
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
      user_name: "test_user".to_string(),
      role: Role::Admin,
    };
    let rto: CreatedRto = user.clone().into();
    assert_eq!(rto.uuid, user.uuid);
  }

}
