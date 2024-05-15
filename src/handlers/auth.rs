use std::borrow::Borrow;

use actix_web::{web, HttpResponse};
use argon2::{Argon2, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::{
    config::{JwtConfig, ServerConfig},
    errors::ApiError,
    schema::users::dsl::*,
    DbPool,
};
use diesel::prelude::*;

use super::{Claims, User};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct LoginResponse {
    pub token: String,
}

pub(crate) async fn token(
    db: web::Data<DbPool>,
    cfg: web::Data<&'static ServerConfig>,
    credentials: web::Json<LoginRequest>,
) -> web::Either<HttpResponse, ApiError> {
    let user = match authenticate_user(db, credentials.into_inner()).await {
        Ok(user) => user,
        Err(e) => return web::Either::Right(e),
    };
    let token = match generate_jwt_token(&user, cfg.jwt.borrow()) {
        Ok(token) => token,
        Err(e) => return web::Either::Right(e),
    };
    web::Either::Left(HttpResponse::Created().json(LoginResponse { token }))
}

async fn authenticate_user(
    db: web::Data<DbPool>,
    credentials: LoginRequest,
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
        &EncodingKey::from_secret(jwt_cfg.secret.as_ref()),
    )?;
    Ok(token)
}
