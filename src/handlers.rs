use actix_web::{post, web, HttpResponse, Responder};

use crate::{
    db,
    models::users::{InsertableUser, LoginFields},
    services::{self, jwt::RefreshInfo},
};

#[post("/token")]
async fn login(
    pool: web::Data<db::DbPool>,
    secret_key: web::Data<String>,
    req_body: web::Json<LoginFields>,
) -> impl Responder {
    let login_fields = req_body.into_inner();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user = match services::users::validate_user(&mut conn, login_fields) {
        Ok(user) => user,
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };

    let jwt = match services::jwt::generate_jwt(&mut conn, secret_key.get_ref().clone(), user) {
        Ok(e) => e,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(jwt)
}

#[post("/register")]
async fn register(
    pool: web::Data<db::DbPool>,
    req_body: web::Json<InsertableUser>,
) -> impl Responder {
    let user_details = req_body.into_inner();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let _user = match services::users::create_user(&mut conn, user_details) {
        Ok(u) => u,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Created().body("User created successfully")
}

#[post("/token/refresh")]
async fn refresh(
    pool: web::Data<db::DbPool>,
    secret_key: web::Data<String>,
    req_body: web::Json<RefreshInfo>,
) -> impl Responder {
    let refresh_info = req_body.into_inner();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let refreshed_tokens = match services::jwt::refresh_token(
        &mut conn,
        secret_key.get_ref().clone(),
        &refresh_info.refresh_token,
    ) {
        Ok(jwt) => jwt,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(refreshed_tokens)
}
