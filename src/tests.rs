#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test, web, App};
    use diesel::prelude::*;
    use dotenvy::dotenv;
    use once_cell::sync::Lazy;
    use serde_json::{json, Value};
    use std::{fs, path::Path};
    use uuid::Uuid;

    // Import your handlers, db module, DbPool type, and schema
    use crate::{
        db::{self, DbPool},
        handlers::{get_email_by_user_id, get_jwks, obtain, refresh, register, revoke},
        models, // For models::users::User
        schema, // For schema::users, schema::refresh_tokens
    };

    // --- Shared Test Resources ---

    const TEST_PEM_KEY_PATH: &str = "keys/dummy.pem"; // PLEASE ADJUST IF YOUR PATH IS DIFFERENT

    static TEST_PEM_KEY: Lazy<String> = Lazy::new(|| {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let key_path = Path::new(manifest_dir).join(TEST_PEM_KEY_PATH);
        fs::read_to_string(&key_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read PEM key from {}: {}. Ensure the file exists and is accessible.",
                key_path.display(),
                e
            )
        })
    });

    static TEST_POOL: Lazy<DbPool> = Lazy::new(|| {
        dotenv().ok();
        db::get_pool().expect("Failed to create shared test database pool")
    });

    // --- Cleanup Helper Functions ---

    /// Deletes a user by email and their associated refresh tokens.
    fn cleanup_user_and_tokens_by_email(email_to_delete: &str) {
        use schema::refresh_tokens::dsl as rt_dsl;
        use schema::users::dsl as users_dsl;

        let mut conn = TEST_POOL
            .get()
            .expect("Failed to get DB connection from pool for cleanup");

        // Find the user to get their ID for deleting associated refresh tokens
        // The User model has `id: Uuid` and the refresh_tokens table has `user_id: Varchar`.
        // We assume user_id in refresh_tokens is the string representation of the User's Uuid.
        let user_result = users_dsl::users
            .filter(users_dsl::email.eq(email_to_delete))
            .select(models::users::User::as_select()) // Selects the whole User struct
            .first::<models::users::User>(&mut conn)
            .optional() // Use optional to avoid panic if user not found (e.g., test failed before creation)
            .expect("DB error while fetching user for cleanup");

        if let Some(user) = user_result {
            let user_id_str = user.id.to_string();
            // Delete refresh tokens associated with the user
            diesel::delete(rt_dsl::refresh_tokens.filter(rt_dsl::user_id.eq(user_id_str)))
                .execute(&mut conn)
                .expect("Failed to delete refresh tokens during cleanup");
        }

        // Delete the user
        let _deleted_users_count =
            diesel::delete(users_dsl::users.filter(users_dsl::email.eq(email_to_delete)))
                .execute(&mut conn)
                .expect("Failed to delete user during cleanup");
    }

    // --- Test Functions (with cleanup) ---

    #[actix_web::test]
    async fn test_register_pacilian_endpoint() {
        let pool = TEST_POOL.clone();
        let test_email = "test_pacilian_reg@example.com";

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(web::scope("/api").service(register)),
        )
        .await;

        let user_payload = json!({
            "email": test_email,
            "password": "password123",
            "role": "pacilian"
        });

        let req = test::TestRequest::post()
            .uri("/api/register")
            .set_json(&user_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::CREATED,
            "Registration for 'pacilian' failed."
        );

        if resp.status() == StatusCode::CREATED {
            let body_bytes = test::read_body(resp).await;
            assert_eq!(body_bytes, "User created successfully");
        }
        cleanup_user_and_tokens_by_email(test_email);
    }

    #[actix_web::test]
    async fn test_register_caregiver_endpoint() {
        let pool = TEST_POOL.clone();
        let test_email = "test_caregiver_reg@example.com";

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(web::scope("/api").service(register)),
        )
        .await;

        let user_payload = json!({
            "email": test_email,
            "password": "password123",
            "role": "caregiver"
        });

        let req = test::TestRequest::post()
            .uri("/api/register")
            .set_json(&user_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::CREATED,
            "Registration for 'caregiver' failed."
        );

        cleanup_user_and_tokens_by_email(test_email);
    }

    #[actix_web::test]
    async fn test_login_endpoint_pacilian() {
        let pool = TEST_POOL.clone();
        let secret_key_for_test = TEST_PEM_KEY.clone();
        let user_email = "login_pacilian_cl@example.com";
        let user_password = "password123";
        let user_role_for_registration = "pacilian";

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(secret_key_for_test))
                .service(web::scope("/api").service(register).service(obtain)),
        )
        .await;

        let register_payload = json!({
            "email": user_email,
            "password": user_password,
            "role": user_role_for_registration
        });
        let reg_req = test::TestRequest::post()
            .uri("/api/register")
            .set_json(&register_payload)
            .to_request();
        test::call_service(&app, reg_req).await; // Assume registration is fine

        let login_payload = json!({ "email": user_email, "password": user_password });
        let login_req = test::TestRequest::post()
            .uri("/api/token/obtain")
            .set_json(&login_payload)
            .to_request();
        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), StatusCode::OK);

        cleanup_user_and_tokens_by_email(user_email);
    }

    #[actix_web::test]
    async fn test_login_endpoint_caregiver() {
        let pool = TEST_POOL.clone();
        let secret_key_for_test = TEST_PEM_KEY.clone();
        let user_email = "login_caregiver_cl@example.com";
        let user_password = "password123";
        let user_role_for_registration = "caregiver";

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(secret_key_for_test))
                .service(web::scope("/api").service(register).service(obtain)),
        )
        .await;

        let register_payload = json!({"email": user_email, "password": user_password, "role": user_role_for_registration});
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/register")
                .set_json(&register_payload)
                .to_request(),
        )
        .await;

        let login_payload = json!({ "email": user_email, "password": user_password });
        let login_req = test::TestRequest::post()
            .uri("/api/token/obtain")
            .set_json(&login_payload)
            .to_request();
        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), StatusCode::OK);

        cleanup_user_and_tokens_by_email(user_email);
    }

    #[actix_web::test]
    async fn test_refresh_token_endpoint() {
        let pool = TEST_POOL.clone();
        let secret_key_for_test = TEST_PEM_KEY.clone();
        let user_email = "refresh_cl@example.com";
        let user_password = "password123";
        let user_role_for_registration = "pacilian";

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(secret_key_for_test))
                .service(
                    web::scope("/api")
                        .service(register)
                        .service(obtain)
                        .service(refresh),
                ),
        )
        .await;

        let register_payload = json!({"email": user_email, "password": user_password, "role": user_role_for_registration});
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/register")
                .set_json(&register_payload)
                .to_request(),
        )
        .await;

        let login_payload = json!({ "email": user_email, "password": user_password });
        let login_req = test::TestRequest::post()
            .uri("/api/token/obtain")
            .set_json(&login_payload)
            .to_request();
        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), StatusCode::OK);

        let login_body_bytes = test::read_body(login_resp).await;
        let jwt_response: Value = serde_json::from_slice(&login_body_bytes).unwrap();
        let refresh_token_str = jwt_response.get("refresh").unwrap().as_str().unwrap();

        let refresh_payload = json!({ "refresh_token": refresh_token_str });
        let refresh_req = test::TestRequest::post()
            .uri("/api/token/refresh")
            .set_json(&refresh_payload)
            .to_request();
        let refresh_resp = test::call_service(&app, refresh_req).await;
        assert_eq!(refresh_resp.status(), StatusCode::OK);

        cleanup_user_and_tokens_by_email(user_email);
    }

    #[actix_web::test]
    async fn test_revoke_token_endpoint() {
        let pool = TEST_POOL.clone();
        let secret_key_for_test = TEST_PEM_KEY.clone();
        let user_email = "revoke_cl@example.com";
        let user_password = "password123";
        let user_role_for_registration = "caregiver";

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(secret_key_for_test))
                .service(
                    web::scope("/api")
                        .service(register)
                        .service(obtain)
                        .service(revoke),
                ),
        )
        .await;

        let register_payload = json!({"email": user_email, "password": user_password, "role": user_role_for_registration});
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/register")
                .set_json(&register_payload)
                .to_request(),
        )
        .await;

        let login_payload = json!({ "email": user_email, "password": user_password });
        let login_req = test::TestRequest::post()
            .uri("/api/token/obtain")
            .set_json(&login_payload)
            .to_request();
        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), StatusCode::OK);

        let login_body_bytes = test::read_body(login_resp).await;
        let jwt_response: Value = serde_json::from_slice(&login_body_bytes).unwrap();
        let refresh_token_str = jwt_response.get("refresh").unwrap().as_str().unwrap();

        let revoke_payload = json!({ "refresh_token": refresh_token_str });
        let revoke_req = test::TestRequest::post()
            .uri("/api/token/revoke")
            .set_json(&revoke_payload)
            .to_request();
        let revoke_resp = test::call_service(&app, revoke_req).await;
        assert_eq!(revoke_resp.status(), StatusCode::OK);

        let revoke_body_bytes = test::read_body(revoke_resp).await;
        assert_eq!(revoke_body_bytes, "Token successfully revoked");

        cleanup_user_and_tokens_by_email(user_email);
    }

    // Tests for get_email_by_user_id and get_jwks do not create persistent user data,
    // so they don't strictly need this type of cleanup unless they were to register users.
    // The get_email_by_user_id test currently checks for non-existent/invalid UUIDs.

    #[actix_web::test]
    async fn test_get_email_by_user_id_endpoint() {
        let pool = TEST_POOL.clone();
        // For this test to check a successful case, a user would need to be created.
        // If so, that user should be cleaned up. For now, it tests error cases.
        let app = test::init_service(
            App::new().app_data(web::Data::new(pool)).service(
                web::scope("/api")
                    .service(register)
                    .service(get_email_by_user_id),
            ),
        )
        .await;

        let invalid_uuid_req = test::TestRequest::get()
            .uri("/api/email/not-a-valid-uuid")
            .to_request();
        let resp_invalid_uuid = test::call_service(&app, invalid_uuid_req).await;
        assert_eq!(
            resp_invalid_uuid.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let random_valid_uuid = Uuid::new_v4().to_string();
        let non_existent_uuid_req = test::TestRequest::get()
            .uri(&format!("/api/email/{}", random_valid_uuid))
            .to_request();
        let resp_non_existent_uuid = test::call_service(&app, non_existent_uuid_req).await;
        assert_eq!(
            resp_non_existent_uuid.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[actix_web::test]
    async fn test_get_jwks_endpoint() {
        let app =
            test::init_service(App::new().service(web::scope("/.well-known").service(get_jwks)))
                .await;
        let req = test::TestRequest::get()
            .uri("/.well-known/jwks.json")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
