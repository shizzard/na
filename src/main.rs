//! The entry point for the web service.
//! TODO: explain the REST API contract

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unused_crate_dependencies,
    unused_results
)]

mod config;
mod errors;
mod handlers;
mod models;
mod schema;

use actix_web::{error::*, web, App, HttpResponse, HttpServer};
use config::ServerConfig;
use diesel::{r2d2::ConnectionManager, PgConnection};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let cfg = config::get_leaked().await;

    start_http_listener(cfg).await
}

pub(crate) type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn start_http_listener(cfg: &'static ServerConfig) -> std::io::Result<()> {
    let manager = ConnectionManager::<PgConnection>::new(&cfg.database.url);
    let db_pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind_addr = cfg.http.as_bind_str();
    log::info!("Starting REST API listener on {bind_addr}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(cfg))
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        let error_reason = err.to_string();
                        InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json(errors::ErrorPayload {
                                reason: error_reason.as_str(),
                            }),
                        )
                        .into()
                    }),
            )
            .service(web::resource("/user").route(web::post().to(handlers::user::register)))
            .service(web::resource("/auth/token").route(web::post().to(handlers::auth::token)))
            .service(web::resource("/users").route(web::get().to(handlers::users::list)))
    })
    .bind(cfg.http.as_bind_str())?
    .run()
    .await
}
