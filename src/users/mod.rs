pub mod dto;
pub mod rto;

use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use dto::create_user_dto::CreateUserDTO;
use dto::get_user_dto::GetUserDTO;
use jsonwebtoken::{decode, DecodingKey, Validation};
use nanoid::nanoid;
use rto::create_user_rto::CreateUserRTO;
use rto::get_user_rto::GetUserRTO;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::shared::model::user::User;
use crate::shared::repository::user_repository::UserRepository;
use crate::shared::role::Role;
use crate::AppState;

pub async fn get_user<UR: UserRepository + 'static>(
  data: web::Data<AppState<UR>>,
  path: web::Path<GetUserDTO>,
  request: HttpRequest,
) -> impl Responder {
  // Perform validation
  if let Err(validation_errors) = path.validate() {
    // If validation fails, return a 400 error with details
    return HttpResponse::BadRequest().json(validation_errors);
  }

  // User from the JWT. Needed to verify if the user has permission to access the user
  // from the query below.
  let auth = find_auth_user::<UR>(request, &data).await;
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
    .json(GetUserRTO::from(user))
}

pub async fn create_user<UR: UserRepository + 'static>(
  data: web::Data<AppState<UR>>,
  payload: web::Json<CreateUserDTO>,
  request: HttpRequest,
) -> impl Responder {

  // User from the JWT. Needed to verify if the user has permission to access the user
  // from the query below.
  let auth = find_auth_user::<UR>(request, &data).await;
  if auth.is_none() {
    return invalid_auth_header();
  }
  let auth = auth.unwrap();
  if auth.role != Role::Admin && auth.role != Role::Manager {
    return HttpResponse::Forbidden().body("Forbidden");
  }

  data
    .user_repository
    .create(User::from(payload.0))
    .await
    .map(|user| {
      HttpResponse::Created()
        .content_type("application/json")
        .append_header((header::LOCATION, format!("/v1/users/{}", user.uuid)))
        .json(CreateUserRTO::from(user))
    })
    .unwrap_or_else(|error| {
      println!("{}", error);
      HttpResponse::InternalServerError().finish()
    })
  
}

impl From<CreateUserDTO> for User {
  fn from(dto: CreateUserDTO) -> Self {
    Self {
      uuid: nanoid!(),
      user_name: dto.user_name,
      role: dto.role
    }
  }
}

// TODO: shouldn't this be an middleware?
fn invalid_auth_header() -> HttpResponse {
  HttpResponse::BadRequest()
    .content_type("application/json")
    .body(r#"{"message": "Invalid Authorization header"}"#)
}

fn user_not_found() -> HttpResponse {
  HttpResponse::NotFound()
    .content_type("application/json")
    .body(r#"{"message": "User not found"}"#)
}

async fn find_auth_user<UR: UserRepository + 'static>(
  request: HttpRequest,
  data: &web::Data<AppState<UR>>,
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
    &DecodingKey::from_secret(data.config.jwt_secret.as_bytes()),
    &Validation::default(),
  );

  if decode_result.is_err() {
    return None;
  }
  let decode_result = decode_result.unwrap();

  Some(decode_result.claims)
}

// Transform User domain to RTO
impl From<User> for GetUserRTO {
  fn from(user: User) -> Self {
    Self {
      uuid: user.uuid,
      user_name: user.user_name,
      role: user.role,
    }
  }
}

// Transform User domain to RTO
impl From<User> for CreateUserRTO {
  fn from(user: User) -> Self {
    Self {
      uuid: user.uuid
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
  uuid: String,
  role: Role,
  exp: usize,
  iat: usize,
}

// impl User {
//   pub fn is_user_allowed(&self, user: &User) -> bool {
//     if self.uuid == user.uuid {
//       return true;
//     }
//     if self.role == Role::Admin {
//       return true;
//     }
//     // TODO: Handle other cases, role based.
//     false
//   }
// }

impl AccessTokenClaims {
  pub fn is_user_allowed(&self, user: &User) -> bool {
    if self.uuid == user.uuid {
      return true;
    }
    if self.role == Role::Admin || self.role == Role::Manager {
      return true;
    }
    // TODO: Handle other cases, role based.
    false
  }
}
