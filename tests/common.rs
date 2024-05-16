use actix_web::{dev::ServiceResponse, error::InternalError, test, web, App, HttpResponse};
use diesel::{r2d2::ConnectionManager, PgConnection};
use na::{config::ServerConfig, errors, handlers, middleware::jwt::JwtMiddleware, DbPool};

pub async fn setup_server() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = ServiceResponse,
    Error = actix_web::Error,
> {
    let cfg = ServerConfig::new_leaked();
    let manager = ConnectionManager::<PgConnection>::new(&cfg.database.url);
    let db_pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    test::init_service(
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
            ),
    )
    .await
}

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn random_string(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
