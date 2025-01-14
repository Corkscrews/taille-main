#[cfg(test)]
pub mod tests {
  use crate::{
    custom_nanoid, shared::role::Role, users::model::access_token_claims::AccessTokenClaims
  };
  use actix_web::{
    http::{header::HeaderValue, StatusCode},
    HttpRequest, Responder,
  };
  use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
  use serde::de::DeserializeOwned;

  pub fn create_fake_access_token_claims() -> AccessTokenClaims {
    AccessTokenClaims {
      uuid: custom_nanoid(),
      role: Role::Manager,
      iat: 0,
      exp: 253402300799,
    }
  }

  pub fn create_fake_access_token(jwt_secret: &str) -> String {
    encode(
      &Header::new(Algorithm::HS256),
      &create_fake_access_token_claims(),
      &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap()
  }

  pub fn http_request(jwt_secret: &str) -> HttpRequest {
    let authorization_header = HeaderValue::from_str(&format!(
      "Bearer {}",
      create_fake_access_token(jwt_secret)
    ))
    .unwrap();
    actix_web::test::TestRequest::default()
      .append_header((
        actix_web::http::header::AUTHORIZATION,
        authorization_header.clone(),
      ))
      .append_header((
        actix_web::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      ))
      .to_http_request()
  }

  pub async fn parse_http_response<T: DeserializeOwned>(
    responder: impl Responder,
    request: &HttpRequest,
    status_code: StatusCode,
  ) -> T {
    // Convert the `Responder` into an HttpResponse
    let http_response = responder.respond_to(request);

    // Wrap the HttpResponse in a ServiceResponse so that test utilities can work with it
    let service_response =
      actix_web::test::TestRequest::default().to_srv_response(http_response);

    let service_status_code = service_response.status();
    // Read the raw body as a string
    let body_bytes = actix_web::test::read_body(service_response).await;
    let body_string = String::from_utf8(body_bytes.to_vec())
      .expect("Response body contains invalid UTF-8");

    // Print the raw string body
    println!("Response Body (String): {}", body_string);

    // Ensure the status matches the expected status_code
    assert_eq!(service_status_code, status_code);

    // Deserialize the string body into the target type T
    serde_json::from_str(&body_string)
      .expect("Failed to deserialize response body")
  }
}
