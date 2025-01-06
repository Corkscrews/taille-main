mod shared;
mod users;
mod config;
mod database;

use std::sync::Arc;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use config::Config;
use database::Database;
use shared::repository::user_repository::{UserRepository, UserRepositoryImpl};
use users::{create_user, get_user};

// This struct represents state
struct AppState<UR: UserRepository> {
  user_repository: UR,
  config: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let server_address = "127.0.0.1:3001";
  println!("Listening on http://{}", server_address);

  let database = Database::new().await;
  let database = Arc::new(database);

  HttpServer::new(move || App::new().configure(|cfg| {
    let user_repository = UserRepositoryImpl::new(database.clone());
    config(cfg, user_repository)
  }))
    .bind(server_address)?
    .run()
    .await
}

// Function to initialize the App
fn config<UR: UserRepository + 'static>(
  config: &mut web::ServiceConfig, 
  user_repository: UR,
) {
  // Rate limit
  // Allow bursts with up to five requests per IP address
  // and replenishes two elements per second
  let governor_config = GovernorConfigBuilder::default()
    .requests_per_second(2)
    .burst_size(5)
    .finish()
    .unwrap();

  config
    .app_data(
      web::Data::new(
        AppState {
          user_repository,
          config: Config::default(),
        }
      )
    )
    .service(
      web::scope("/v1").service(
        web::scope("/users")
          .wrap(Governor::new(&governor_config))
          .route("/{uuid}", web::get().to(get_user::<UR>))
          .route("", web::post().to(create_user::<UR>)),
      ),
    );
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::{http::header::HeaderValue, test, App};
  use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
  use serde::{Deserialize, Serialize};
  use shared::{repository::user_repository::tests::UserRepositoryMock, role::Role};
  use users::rto::{create_user_rto::CreateUserRTO, get_user_rto::GetUserRTO};
  use std::{env, net::SocketAddr, str::FromStr};
  use nanoid::nanoid;

  #[actix_rt::test]
  async fn test_get_user_in_memory() {
    let master_key = String::from("FAKE_MASTER_KEY");
    let jwt_secret = String::from("FAKE_JWT_SECRET");
    env::set_var("MASTER_KEY", &master_key);
    env::set_var("JWT_SECRET", "FAKE_JWT_SECRET");

    // Initialize the service in-memory
    let app = test::init_service(
      App::new().configure(|cfg| {
        let user_repository = UserRepositoryMock::new();
        config(cfg, user_repository)
      }), // your config function
    )
    .await;

    let authorization_header = HeaderValue::from_str(
      &format!(
        "Bearer {}",
        create_fake_access_token(&jwt_secret)
      )
    ).unwrap();

    // 1) Create user
    let create_req = test::TestRequest::post()
      .uri("/v1/users")
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        authorization_header.clone()
      ))
      .append_header((
        actix_web::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      ))
      .set_json(serde_json::json!({
          "userName": "testuser",
          "password": "testpassword",
          "role": Role::Driver
      }))
      .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    println!("{:?}", create_resp.response().body());
    assert!(create_resp.status().is_success(), "Create user failed");

    // Use the Actix Web test helper to read the response body
    let create_body_bytes = test::read_body(create_resp).await;
    let create_body_str = std::str::from_utf8(&create_body_bytes)
      .expect("Response body should be valid UTF-8");

    // Deserialize the JSON response into your struct
    let create_user_rto: CreateUserRTO = serde_json::from_str(create_body_str)
      .expect("Failed to parse response JSON");

    // 2) Get user
    let get_user_req = test::TestRequest::get()
      .uri(&format!("/v1/users/{}", create_user_rto.uuid))
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        authorization_header
      ))
      .append_header((
        actix_web::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      ))
      .to_request();

    let get_user_resp = test::call_service(&app, get_user_req).await;
    assert!(get_user_resp.status().is_success(), "Get user failed");

    // Use the Actix Web test helper to read the response body
    let get_user_body_bytes = test::read_body(get_user_resp).await;
    let get_user_body_str = std::str::from_utf8(&get_user_body_bytes)
      .expect("Response body should be valid UTF-8");

    // Deserialize the JSON response into your struct
    let get_user_rto: GetUserRTO = serde_json::from_str(get_user_body_str)
      .expect("Failed to parse response JSON");
    assert_eq!(get_user_rto.uuid, create_user_rto.uuid);
  }

  #[derive(Serialize, Deserialize)]
  struct FakeAccessTokenClaims {
    uuid: String,
    role: Role,
    sub: String,
    iat: u64,
    exp: u64,
  }

  fn create_fake_access_token(jwt_secret: &str) -> String {
    let fake_claims = FakeAccessTokenClaims {
      uuid: nanoid!(),
      role: Role::Manager,
      sub: String::from("foo"),
      iat: 0,
      exp: 253402300799,
    };
    encode(
      &Header::new(Algorithm::HS256),
      &fake_claims,
      &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap()
  }
}
