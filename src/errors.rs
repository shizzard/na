use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
#[error("Api handler failed")]
pub struct ApiError {
    inner: Box<dyn std::error::Error>,
    pub response: HttpResponse,
}

use diesel::result::Error as DieselError;
impl From<DieselError> for ApiError {
    fn from(value: DieselError) -> Self {
        Self {
            inner: Box::new(value),
            response: HttpResponse::InternalServerError().json(ErrorPayload {
                reason: "Internal server error",
            }),
        }
    }
}

use argon2::password_hash::errors::Error as Argon2Error;
impl From<Argon2Error> for ApiError {
    fn from(value: Argon2Error) -> Self {
        Self {
            inner: Box::new(value),
            response: HttpResponse::InternalServerError().json(ErrorPayload {
                reason: "Internal server error",
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload<'a> {
    pub reason: &'a str,
}
