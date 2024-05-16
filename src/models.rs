use crate::schema::users::dsl::users as users_dsl;
use crate::{errors::ApiError, schema::*, DbPool};
use actix_web::web;
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use serde::Deserialize;

///
/// Data structure representing the registered user.
///
/// This struct is not serializable for a reason: it contains sensitive data
/// and we don't want this data to pass outside of the system.
/// If you want to serialize the data to pass it somewhere, use a separate
/// data structure (see crate::handlers::OutputUser for example).
#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub hashed_password: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

///
/// Data structure representing the registering (but not yet registered) user.
///
/// This struct is not serializable for a reason: it contains sensitive data
/// and we don't want this data to pass outside of the system.
/// If you want to serialize the data to pass it somewhere, use a separate
/// data structure (see crate::handlers::OutputUser for example).
#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub name: String,
    pub hashed_password: String,
}

impl NewUser {
    /// Write a new user to the database.
    /// Executes a database query, so it must be wrapped with actix' `web::block`.
    pub fn write(&self, db: web::Data<DbPool>) -> Result<User, ApiError> {
        let mut conn = db.get()?;
        let inserted_user = diesel::insert_into(users_dsl)
            .values(self)
            .get_result(&mut conn)?;

        Ok(inserted_user)
    }
}
