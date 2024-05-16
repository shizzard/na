use actix_web::{web, HttpResponse};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::{errors::ApiError, DbPool};

use super::OutputUser;
use crate::models::{NewUser, User};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct InputUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

///
/// Register a new user endpoint.
///
/// Accepts three parameters:
/// - name: string
/// - email: string
/// - password: string
///
/// Returns a created user record.
///
/// POST /user
/// Example:
/// {
///   "name": "John",
///   "email": "john@example.org",
///   "password": "secr3t"
/// }'
///
/// Returns
/// {
///   "created_at": "2024-05-16T10:25:41.800997",
///   "email": "john@example.org",
///   "id": 9,
///   "name": "john"
/// }
pub(crate) async fn register(
    db: web::Data<DbPool>,
    item: web::Json<InputUser>,
) -> web::Either<HttpResponse, ApiError> {
    match register_single_user(db, item.into_inner()).await {
        Ok(user) => web::Either::Left(HttpResponse::Created().json(OutputUser::from(user))),
        Err(e) => {
            log::warn!("Cannot register the user: {}", e);
            web::Either::Right(e)
        }
    }
}

async fn register_single_user(db: web::Data<DbPool>, item: InputUser) -> Result<User, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(item.password.as_bytes(), &salt)?
        .to_string();

    let new_user = NewUser {
        name: item.name,
        email: item.email,
        hashed_password: hash,
    };

    match web::block(move || new_user.write(db)).await? {
        Ok(user) => Ok(user),
        Err(e) => Err(e),
    }
}
