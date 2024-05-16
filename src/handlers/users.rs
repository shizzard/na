//!
//! Handler for handling users listing requests.
//!
//! Relies on JWT middleware to ensure authorization.

use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel::ExpressionMethods;

use crate::schema::users::dsl::*;
use crate::{errors::ApiError, models::User, DbPool};

use super::OutputUser;

// Might be put into server config
const DEFAULT_QUERY_LIMIT: i32 = 10;

/// Users list request representation.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ListRequest {
    /// A number of user records to retrieve (optional, default: 10)
    pub limit: Option<i32>,
    /// User id to start the list from (not inclusive, optional, default: 0)
    pub after: Option<i32>,
}

/// Users list response representation.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ListResponse {
    /// A list of user records (see [OutputUser]).
    pub users: Vec<OutputUser>,
}

///
/// Get list of registered users endpoint.
///
/// Accepts [ListRequest].
/// Requires Authorization via JWT (see /auth/token handler).
/// Returns [ListResponse].
///
/// Example:
/// GET /users?limit=5&after=16
/// Authorization: Bearer [token]
///
/// Returns (assuming only two records left)
/// {
///   "users": [
///     {
///       "created_at": "2024-05-15T19:49:55.314405",
///       "email": "john@exmaple.org",
///       "id": 17,
///       "name": "John"
///     },
///     {
///       "created_at": "2024-05-15T19:50:05.008961",
///       "email": "mary@example.org",
///       "id": 18,
///       "name": "Mary"
///     }
///   ]
/// }
///
pub async fn list(
    db: web::Data<DbPool>,
    query: web::Query<ListRequest>,
) -> web::Either<HttpResponse, ApiError> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(DEFAULT_QUERY_LIMIT);
    let after = query.after.unwrap_or(0);

    match load_users(db, limit, after).await {
        Ok(user_list) => {
            let response = HttpResponse::Ok().json(ListResponse {
                users: user_list.into_iter().map(OutputUser::from).collect(),
            });
            web::Either::Left(response)
        }
        Err(e) => web::Either::Right(e),
    }
}

async fn load_users(db: web::Data<DbPool>, limit: i32, after: i32) -> Result<Vec<User>, ApiError> {
    web::block(move || -> Result<Vec<User>, ApiError> {
        let mut conn = db.get()?;
        users
            .limit(limit as i64)
            .order_by(id)
            .filter(id.gt(after))
            .load::<User>(&mut conn)
            .map_err(ApiError::from)
    })
    .await?
}
