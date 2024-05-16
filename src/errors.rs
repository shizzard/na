use actix_web::HttpResponse;
use actix_web::{error::BlockingError as ActixBlockingError, Responder};
use argon2::password_hash::errors::Error as Argon2Error;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use jsonwebtoken::errors::Error as JwtError;
use r2d2::Error as R2d2Error;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Database error: {from}")]
    Diesel {
        #[from]
        from: DieselError,
    },
    #[error("Hashing error: {from}")]
    Argon2 {
        #[from]
        from: Argon2Error,
    },
    #[error("Actix blocking operation error: {from}")]
    ActixBlocking {
        #[from]
        from: ActixBlockingError,
    },
    #[error("Database pool error: {from}")]
    R2d2 {
        #[from]
        from: R2d2Error,
    },
    #[error("JWT error: {from}")]
    Jwt {
        #[from]
        from: JwtError,
    },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload<'a> {
    pub reason: &'a str,
}
