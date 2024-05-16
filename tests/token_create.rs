mod common;

use actix_web::test;
use na::handlers::{auth::TokenCreateRequest, user::InputUser};

/// Checks if token can be created
#[actix_web::test]
async fn create_token() {
    let app = common::setup_server().await;
    let email = common::random_string(16);
    let password = common::random_string(16);

    let req = test::TestRequest::post()
        .uri("/user")
        .set_json(InputUser {
            name: common::random_string(16),
            email: email.clone(),
            password: password.clone(),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(201, resp.status().as_u16());

    let req = test::TestRequest::post()
        .uri("/auth/token")
        .set_json(TokenCreateRequest {
            email: email.clone(),
            password: password.clone(),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(201, resp.status().as_u16());
}

/// Checks if service don't create a token if password is invalid.
#[actix_web::test]
async fn create_token_invalid_password() {
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
        .uri("/auth/token")
        .set_json(TokenCreateRequest {
            email: email.clone(),
            password: common::random_string(16),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(400, resp.status().as_u16());
}
