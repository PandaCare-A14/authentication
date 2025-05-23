use std::fs;
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4};

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;

mod db;
mod errors;
mod handlers;
mod models;
mod repository;
mod schema;
mod services;

#[cfg(test)]
mod tests;

use crate::handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::middleware::Logger;
    use env_logger;
    use log;

    match dotenv() {
        Ok(_) => {}
        Err(_err) => {}
    };

    let pool = db::get_pool().unwrap();

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let secret_key: String = fs::read_to_string("keys/rsa-private.pem")
        .map_err(|err| Error::new(ErrorKind::NotFound, err.to_string()))?;

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .map_err(|err| Error::new(ErrorKind::NotFound, err.to_string()))?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(secret_key.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .service(obtain)
                    .service(register)
                    .service(refresh)
                    .service(revoke)
                    .service(get_email_by_user_id),
            )
            .service(web::scope("/.well-known").service(get_jwks))
    })
    .bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))?
    .run()
    .await
}
