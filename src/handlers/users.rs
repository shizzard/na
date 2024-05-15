use actix_web::{web, Responder};

use crate::DbPool;

pub(crate) async fn list(_db: web::Data<DbPool>) -> impl Responder {
    "user list"
}
