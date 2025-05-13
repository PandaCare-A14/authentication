use actix_web::{test, App};
use dotenvy::dotenv;
use serde_json::json;

// Import your handlers
use crate::handlers::{obtain, refresh, register};
// Import your database pool helper
use crate::db;

#[actix_web::test]
async fn test_register_endpoint() {
    dotenv().ok();
    let pool = db::get_pool().expect("Failed to get pool");
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(pool))
            .service(register),
    )
    .await;

    let req_body = json!({
        "email": "testuser@example.com",
        "password": "password123",
        "name": "Test User",
        "nik": "3172011309050003",
        "phone_number": "8557185883"
    });

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let resp_body = test::read_body(resp).await;
    assert_eq!(resp_body, "User created successfully");
}

#[actix_web::test]
async fn test_login_endpoint() {
    dotenv().ok();
    let pool = db::get_pool().expect("Failed to get pool");

    // Build an application with both register and login endpoints
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .service(register)
            .service(obtain),
    )
    .await;

    // First, register a user that we can later log in
    let register_body = json!({
        "email": "loginuser@example.com",
        "password": "password123",
        "name": "Login User",
        "nik": "3172011309050004",
        "phone_number": "8557185884"
    });
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Now, log in with the created user
    let login_body = json!({
        "email": "loginuser@example.com",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let jwt: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON");
    assert!(jwt.get("access").is_some());
    assert!(jwt.get("refresh").is_some());
}

#[actix_web::test]
async fn test_refresh_endpoint() {
    dotenv().ok();
    let pool = db::get_pool().expect("Failed to get pool");

    // Build application with register, login, and refresh endpoints
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .service(register)
            .service(obtain)
            .service(refresh),
    )
    .await;
    // Build application with register, login, and refresh endpoints

    // Register and login to obtain a refresh token
    let register_body = json!({
        "email": "refreshuser@example.com",
        "password": "password123",
        "name": "Refresh User",
        "nik": "3172011309050005",
        "phone_number": "8557185885"
    });
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let login_body = json!({
        "email": "refreshuser@example.com",
        "password": "password123"
    });
    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let login_resp_body = test::read_body(resp).await;
    let jwt: serde_json::Value = serde_json::from_slice(&login_resp_body).expect("Invalid JSON");
    let refresh_token = jwt
        .get("refresh")
        .expect("Missing refresh token")
        .as_str()
        .expect("refresh token not a string");

    // Now test token refresh
    let refresh_body = json!({
        "refresh_token": refresh_token
    });
    let req = test::TestRequest::post()
        .uri("/token/refresh")
        .set_json(&refresh_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let refresh_resp_body = test::read_body(resp).await;
    let new_jwt: serde_json::Value =
        serde_json::from_slice(&refresh_resp_body).expect("Invalid JSON");
    assert!(new_jwt.get("access").is_some());
}
