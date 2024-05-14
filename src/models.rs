use crate::schema::*;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

//
// This struct is not serializable for a reason: it contains sensitive data
// and we don;t want this data to pass outside of the system.
// If you want to serialize the data to pass it somewhere, use a separate
// data structure (see crate::handlers::OutputUser for example).
#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub hashed_password: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub name: &'a str,
    pub hashed_password: &'a str,
}
