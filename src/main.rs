//!
//! The entry point for the web service.
//!
//! The endpoints are:
//! - POST /user: create a new user.
//! - POST /auth/token: crate a new access token
//! - GET /users: get a list of registered users

use actix_web::{error::*, web, App, HttpResponse, HttpServer};
use diesel::{r2d2::ConnectionManager, PgConnection};
use na::config::ServerConfig;
use na::middleware::jwt::JwtMiddleware;
use na::{errors, handlers, DbPool};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let cfg = ServerConfig::new_leaked();

    start_http_listener(cfg).await
}

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
            .service(
                web::resource("/users")
                    .wrap(JwtMiddleware {
                        jwt_config: &cfg.jwt,
                    })
                    .route(web::get().to(handlers::users::list)),
            )
    })
    .bind(cfg.http.as_bind_str())?
    .run()
    .await
}
