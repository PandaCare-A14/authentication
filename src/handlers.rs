use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::{
    db,
    models::users::{InsertableUser, LoginFields},
    repository::{jwt::revoke_refresh_token, users::get_user_by_id},
    services::{
        self,
        jwt::{RefreshInfo, RevocationInfo},
    },
};

#[post("/token/obtain")]
async fn obtain(
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

#[post("/token/revoke")]
async fn revoke(
    pool: web::Data<db::DbPool>,
    req_body: web::Json<RevocationInfo>,
) -> impl Responder {
    let revocation_info = req_body.into_inner();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if let Err(err) = revoke_refresh_token(&mut conn, &revocation_info.refresh_token) {
        return HttpResponse::NotModified().body(err.to_string());
    };

    HttpResponse::Ok().body("Token successfully revoked")
}

#[get("/jwks.json")]
async fn get_jwks() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("jwks/jwks.json")?)
}

#[get("/email/{user_id}")]
async fn get_email_by_user_id(
    pool: web::Data<db::DbPool>,
    user_id: web::Path<String>,
) -> HttpResponse {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user_id = match Uuid::parse_str(&user_id) {
        Ok(user_id) => user_id,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user = match get_user_by_id(&mut conn, user_id) {
        Ok(user) => user,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(user.email)
}
