mod common;

use actix_web::{dev::ServiceResponse, http, test};
use na::handlers::{
    auth::{TokenCreateRequest, TokenCreateResponse},
    user::InputUser,
    users::ListResponse,
    OutputUser,
};
use serial_test::serial;

async fn create_random_user(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) {
    let req = test::TestRequest::post()
        .uri("/user")
        .set_json(InputUser {
            name: common::random_string(16),
            email: common::random_string(16),
            password: common::random_string(16),
        })
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(201, resp.status().as_u16());
}

/// Checks if user list may be retrieved.
/// Since these tests are very naive and do not mock the database, I cannot
/// really check against the list response values here.
#[actix_web::test]
#[serial]
async fn list_users() {
    let app = common::setup_server().await;
    let email = common::random_string(16);
    let password = common::random_string(16);

    // register new user
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

    // get auth token
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .set_json(TokenCreateRequest {
            email: email.clone(),
            password: password.clone(),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(201, resp.status().as_u16());
    let body = test::read_body(resp).await;
    let token_create_response: TokenCreateResponse = serde_json::from_slice(&body).unwrap();

    // retrieve a list of users
    let req = test::TestRequest::get()
        .uri("/users")
        .append_header((
            http::header::AUTHORIZATION,
            format!("Bearer {}", token_create_response.token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(200, resp.status().as_u16());
}

/// Checks if user list with `after` parameter may be retrieved.
#[actix_web::test]
#[serial]
async fn list_users_after() {
    let app = common::setup_server().await;
    let email = common::random_string(16);
    let password = common::random_string(16);

    // register a new user
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
    let body = test::read_body(resp).await;
    let self_user: OutputUser = serde_json::from_slice(&body).unwrap();

    // register five additional users to list
    for _ in 1..=5 {
        create_random_user(&app).await;
    }

    // get auth token
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .set_json(TokenCreateRequest {
            email: email.clone(),
            password: password.clone(),
        })
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(201, resp.status().as_u16());
    let body = test::read_body(resp).await;
    let token_create_response: TokenCreateResponse = serde_json::from_slice(&body).unwrap();

    // list all users after self
    let req = test::TestRequest::get()
        .uri(format!("/users?after={}", self_user.id).as_str())
        .append_header((
            http::header::AUTHORIZATION,
            format!("Bearer {}", token_create_response.token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(200, resp.status().as_u16());
    let body = test::read_body(resp).await;
    let list_response: ListResponse = serde_json::from_slice(&body).unwrap();
    println!("/users?after={}", self_user.id);
    println!("{:?}", list_response);
    assert_eq!(5, list_response.users.len());

    // list all users after self with limit
    let req = test::TestRequest::get()
        .uri(format!("/users?after={}&limit={}", self_user.id, 3).as_str())
        .append_header((
            http::header::AUTHORIZATION,
            format!("Bearer {}", token_create_response.token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(200, resp.status().as_u16());
    let body = test::read_body(resp).await;
    let list_response: ListResponse = serde_json::from_slice(&body).unwrap();
    println!("/users?after={}", self_user.id);
    println!("{:?}", list_response);
    assert_eq!(3, list_response.users.len());
}
