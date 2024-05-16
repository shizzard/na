pub mod auth;
pub mod user;
pub mod users;

use serde::Serialize;

use crate::models::*;

///
/// Generic user data structure to be used for rendering users data as
/// API responses.
///
/// Omits sensitive fields (password and updated_at).
///
/// Should be created from the database-sourced User struct.
#[derive(Debug, Serialize)]
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
