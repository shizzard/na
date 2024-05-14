use actix_web::{web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::errors::ApiError;
use crate::{models::*, schema::users::dsl::*, DbPool};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InputUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OutputUser {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}

impl From<User> for OutputUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
        }
    }
}

pub(crate) async fn user_register(
    db: web::Data<DbPool>,
    item: web::Json<InputUser>,
) -> HttpResponse {
    match add_single_user(db, &item).await {
        Ok(user) => HttpResponse::Created().json(OutputUser::from(user)),
        Err(e) => {
            log::error!("Cannot register the user ({:?}): {}", &item, e);
            e.response
        }
    }
}

async fn add_single_user(
    db: web::Data<DbPool>,
    item: &web::Json<InputUser>,
) -> Result<User, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(item.password.as_bytes(), &salt)?
        .to_string();

    let new_user = NewUser {
        name: &item.name,
        email: &item.email,
        hashed_password: hash.as_str(),
    };

    let mut conn = db.get().expect("Active connection");
    let inserted_user = diesel::insert_into(users)
        .values(&new_user)
        .get_result(&mut conn)?;

    Ok(inserted_user)
}

pub(crate) async fn user_login() -> impl Responder {
    format!("user_login")
}

pub(crate) async fn users_get() -> impl Responder {
    format!("users_get")
}
