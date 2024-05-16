//!
//! Module contains API errors and ways to convert those errors into API responses.

use actix_web::HttpResponse;
use actix_web::{error::BlockingError as ActixBlockingError, Responder};
use argon2::password_hash::errors::Error as Argon2Error;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use jsonwebtoken::errors::Error as JwtError;
use r2d2::Error as R2d2Error;
use serde::{Deserialize, Serialize};

/// Enum representing API errors.
///
/// Implements [Responder] trait for actix_web, and can be used as a return
/// value for request handlers.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    /// Database error representation.
    ///
    /// May contain handlable errors (for example, a duplicate for index).
    #[error("Database error: {from}")]
    Diesel {
        ///
        #[from]
        from: DieselError,
    },
    /// Hashing error representation.
    ///
    /// Irrecoverable.
    #[error("Hashing error: {from}")]
    Argon2 {
        ///
        #[from]
        from: Argon2Error,
    },
    /// Actix blocking operations thread error representation.
    ///
    /// Irrecoverable.
    #[error("Actix blocking operation error: {from}")]
    ActixBlocking {
        ///
        #[from]
        from: ActixBlockingError,
    },
    /// Database pool error representation.
    ///
    /// Irrecoverable.
    #[error("Database pool error: {from}")]
    R2d2 {
        ///
        #[from]
        from: R2d2Error,
    },
    /// JWT error representation.
    ///
    /// May contain handlable errors (for example, JWT token validation error).
    #[error("JWT error: {from}")]
    Jwt {
        ///
        #[from]
        from: JwtError,
    },
    /// Invalid credentials error.
    ///
    /// Specific for JWT token create request.
    #[error("Invalid credentials provided")]
    InvalidCredentials {},
}

impl Responder for ApiError {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            Self::ActixBlocking { .. }
            | Self::Argon2 { .. }
            | Self::R2d2 { .. }
            | Self::Jwt { .. } => {
                // Probably not the best place to put logs into?..
                log::error!(
                    "Responding an error to '{} {}' request due to error: {}",
                    req.method(),
                    req.uri(),
                    &self
                );
                generic_ise()
            }
            Self::InvalidCredentials {} => HttpResponse::BadRequest().json(ErrorPayload {
                reason: "Invalid credentials",
            }),
            Self::Diesel { from } => {
                if let DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = from {
                    return HttpResponse::Conflict().json(ErrorPayload {
                        reason: "Resource already exists",
                    });
                }
                generic_ise()
            }
        }
    }
}

fn generic_ise() -> HttpResponse<<ApiError as Responder>::Body> {
    HttpResponse::InternalServerError().json(ErrorPayload {
        reason: "Internal server error",
    })
}

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload<'a> {
    pub reason: &'a str,
}
