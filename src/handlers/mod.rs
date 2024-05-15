pub mod auth;
pub mod user;
pub mod users;

use serde::{Deserialize, Serialize};

use crate::models::*;

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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    sub: String,
    exp: usize,
}
