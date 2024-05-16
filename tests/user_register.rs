mod common;

use actix_web::test;
use na::handlers::user::InputUser;

/// Checks if user can be registered
#[actix_web::test]
async fn register_user() {
    let app = common::setup_server().await;

    let req = test::TestRequest::post()
        .uri("/user")
        .set_json(InputUser {
            name: common::random_string(16),
            email: common::random_string(16),
            password: common::random_string(16),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(201, resp.status().as_u16());
}

/// Checks if service responds with 409 Conflict when registering the user with
/// the duplicate email.
#[actix_web::test]
async fn register_user_duplicate() {
    let app = common::setup_server().await;
    let email = common::random_string(16);

    let req = test::TestRequest::post()
        .uri("/user")
        .set_json(InputUser {
            name: common::random_string(16),
            email: email.clone(),
            password: common::random_string(16),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(201, resp.status().as_u16());

    let req = test::TestRequest::post()
        .uri("/user")
        .set_json(InputUser {
            name: common::random_string(16),
            email: email.clone(),
            password: common::random_string(16),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(409, resp.status().as_u16());
}
