use std::fs;
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4};

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use jwk_kit::generator::rsa::{extract_rsa_n_e, generate_rsa_keypair_pem};
use jwk_kit::jwk::{create_jwks, JwkBuilder};
use uuid::Uuid;

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

    let (private_key, public_key) = generate_rsa_keypair_pem(2048)
        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;

    // Unwrap: generated RSA key pair should be valid
    let (n_b64, e_b64) = extract_rsa_n_e(&public_key).unwrap();

    let rsa_jwk = JwkBuilder::new("RSA")
        .set_key_use("sig")
        .set_algorithm("RS256")
        .set_key_id(&Uuid::new_v4().to_string())
        .set_modulus(&n_b64)
        .set_exponent(&e_b64)
        .build()
        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;

    let jwks = create_jwks(vec![rsa_jwk]);

    let jwks_string = serde_json::to_string_pretty(&jwks)?;

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .map_err(|err| Error::new(ErrorKind::NotFound, err.to_string()))?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(private_key.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .service(obtain)
                    .service(register)
                    .service(refresh)
                    .service(revoke),
            )
            .service(
                web::scope("/oauth2")
                    .app_data(web::Data::new(jwks_string.clone()))
                    .service(get_jwks),
            )
    })
    .bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))?
    .run()
    .await
}
