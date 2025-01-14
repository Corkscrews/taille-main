use crate::{
  shared::config::Config, users::model::access_token_claims::AccessTokenClaims,
};
use actix_web::web::Data;
use actix_web::Error;
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};

impl FromRequest for AccessTokenClaims {
  type Error = Error;
  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
    let config: &Config = req.app_data::<Data<Config>>().unwrap();
    ready(
      req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
        .and_then(|token| find_auth_user(config, token).ok())
        .ok_or_else(|| {
          actix_web::error::ErrorUnauthorized("Invalid Authorization header")
        }),
    )
  }
}

fn find_auth_user(
  config: &Config,
  token: &str,
) -> Result<AccessTokenClaims, jsonwebtoken::errors::Error> {
  decode::<AccessTokenClaims>(
    token,
    &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
    &Validation::default(),
  )
  .map(|token| token.claims)
}
