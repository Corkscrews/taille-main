mod helpers;
mod shared;
mod trips;
mod users;

use std::sync::Arc;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use shared::config::Config;
use shared::database::Database;
use users::repository::user_repository::{UserRepository, UserRepositoryImpl};
use trips::repository::trip_repository::{TripRepository, TripRepositoryImpl};
use trips::{create_trip, get_trip};
use users::{create_user, get_user};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let server_address = "127.0.0.1:3001";
  println!("Listening on http://{}", server_address);

  std::env::set_var("RUST_LOG", "debug");
  env_logger::init();

  let database = Database::new().await;
  let database = Arc::new(database);

  let user_repository = Arc::new(UserRepositoryImpl::new(database.clone()));
  let trip_repository = Arc::new(TripRepositoryImpl::new(database.clone()));

  HttpServer::new({
    let user_repository = Arc::clone(&user_repository);
    let trip_repository = Arc::clone(&trip_repository);
    move || {
      App::new().configure(|cfg| {
        apply_service_config(cfg, &user_repository, &trip_repository)
      })
    }
  })
  .bind(server_address)?
  .run()
  .await
}

// Function to initialize the App
fn apply_service_config<
  UR: UserRepository + 'static, 
  TR: TripRepository + 'static
>(
  config: &mut web::ServiceConfig,
  user_repository: &Arc<UR>,
  trip_repository: &Arc<TR>
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
    .app_data(web::Data::new(Config::default()))
    .app_data(web::Data::new(user_repository.clone()))
    .app_data(web::Data::new(trip_repository.clone()))
    .service(
      web::scope("/v1")
        .service(
          web::scope("/users")
            .wrap(Governor::new(&governor_config))
            .route("/{uuid}", web::get().to(get_user::<UR>))
            .route("", web::post().to(create_user::<UR>)),
        )
        .service(
          web::scope("/trips")
            .wrap(Governor::new(&governor_config))
            .route("/{uuid}", web::get().to(get_trip::<TR>))
            .route("", web::post().to(create_trip::<TR>)),
        ),
    );
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::{http::header::HeaderValue, test, App};
  use helpers::tests::create_fake_access_token;
  use shared::{role::Role,
    rto::created_rto::CreatedRto,
  };
  use trips::repository::trip_repository::tests::InMemoryTripRepository;
  use std::{env, net::SocketAddr, str::FromStr};
  use users::{repository::user_repository::tests::InMemoryUserRepository, rto::get_user_rto::GetUserRto};

  #[actix_rt::test]
  async fn test_get_user_in_memory() {

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let master_key = String::from("FAKE_MASTER_KEY");
    let jwt_secret = String::from("FAKE_JWT_SECRET");
    env::set_var("MASTER_KEY", &master_key);
    env::set_var("JWT_SECRET", "FAKE_JWT_SECRET");

    let user_repository = Arc::new(InMemoryUserRepository::new());
    let trip_repository = Arc::new(InMemoryTripRepository::new());

    // Initialize the service in-memory
    let app = test::init_service({
      let user_repository = Arc::clone(&user_repository);
      let trip_repository = Arc::clone(&trip_repository);
      App::new().configure(|cfg| {
        apply_service_config(cfg, &user_repository, &trip_repository)
      })
    })
    .await;

    let authorization_header = HeaderValue::from_str(&format!(
      "Bearer {}",
      create_fake_access_token(&jwt_secret)
    ))
    .unwrap();

    // 1) Create user
    let create_req = test::TestRequest::post()
      .uri("/v1/users")
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        authorization_header.clone(),
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
    let create_user_rto: CreatedRto = serde_json::from_str(create_body_str)
      .expect("Failed to parse response JSON");

    // 2) Get user
    let get_user_req = test::TestRequest::get()
      .uri(&format!("/v1/users/{}", create_user_rto.uuid))
      .peer_addr(SocketAddr::from_str("127.0.0.1:12345").unwrap())
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        authorization_header,
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
    let get_user_rto: GetUserRto = serde_json::from_str(get_user_body_str)
      .expect("Failed to parse response JSON");
    assert_eq!(get_user_rto.uuid, create_user_rto.uuid);
  }
}
