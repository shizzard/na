//!
//! Handler for creating JWT tokens.

use std::borrow::Borrow;

use actix_web::{web, HttpResponse};
use argon2::{Argon2, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::{
    config::{JwtConfig, ServerConfig},
    errors::ApiError,
    middleware::jwt::Claims,
    schema::users::dsl::*,
    DbPool,
};
use diesel::prelude::*;

use super::User;

///
/// Token create request representation.
///
/// All fields are optional.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TokenCreateRequest {
    /// Corresponds to the same field in [User] struct.
    pub email: String,
    /// Corresponds to the same field in [User] struct.
    pub password: String,
}

///
/// Token create response representation.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenCreateResponse {
    /// JWT token to be used within `Authorization` HTTP header.
    pub token: String,
}

///
/// Create authorization token endpoint.
///
/// Accepts two parameters:
/// - email: string
/// - password: string
///
/// Returns a JWT auth token.
///
/// Example:
/// POST /auth/token
/// {
///   "email": "john@example.org",
///   "password": "secr3t"
/// }
///
/// Returns
/// {
///     "token": "eyJ0e...xb26ww"
/// }
pub async fn token(
    db: web::Data<DbPool>,
    cfg: web::Data<&'static ServerConfig>,
    credentials: web::Json<TokenCreateRequest>,
) -> web::Either<HttpResponse, ApiError> {
    let user = match authenticate_user(db, credentials.into_inner()).await {
        Ok(user) => user,
        Err(e) => return web::Either::Right(e),
    };
    let token = match generate_jwt_token(&user, cfg.jwt.borrow()) {
        Ok(token) => token,
        Err(e) => return web::Either::Right(e),
    };
    web::Either::Left(HttpResponse::Created().json(TokenCreateResponse { token }))
}

async fn authenticate_user(
    db: web::Data<DbPool>,
    credentials: TokenCreateRequest,
) -> Result<User, ApiError> {
    let user = web::block(move || -> Result<User, ApiError> {
        let mut conn = db.get()?;
        users
            .filter(email.eq(credentials.email))
            .first::<User>(&mut conn)
            .map_err(|_| ApiError::InvalidCredentials {})
    })
    .await??;

    let argon2 = Argon2::default();
    let parsed_hash = argon2::PasswordHash::new(&user.hashed_password)
        .map_err(|_| ApiError::InvalidCredentials {})?;

    argon2
        .verify_password(credentials.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::InvalidCredentials {})?;

    Ok(user)
}

fn generate_jwt_token(user: &User, jwt_cfg: &JwtConfig) -> Result<String, ApiError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("Valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.email.clone(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_cfg.secret.as_bytes()),
    )?;
    Ok(token)
}
