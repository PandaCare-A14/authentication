use actix_web::{post, web, HttpResponse, Responder};

use crate::{
    db,
    interfaces::users::{InsertableUser, LoginFields},
    repository::users::validate_user,
    services,
};

use jsonwebtoken;

#[post("/login")]
async fn login(pool: web::Data<db::DbPool>, req_body: web::Json<LoginFields>) -> impl Responder {
    let login_fields = req_body.into_inner();

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let user = match validate_user(&mut conn, login_fields) {
        Ok(user) => user,
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };

    HttpResponse::NotImplemented().finish()
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

    let user = match services::users::create_user(&mut conn, user_details) {
        Ok(u) => u,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(user)
}
