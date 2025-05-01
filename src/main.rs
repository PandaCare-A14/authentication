use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};

use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use dotenvy::dotenv;
use log;

mod db;
mod errors;
mod handlers;
mod models;
mod repository;
mod schema;
mod services;

use crate::handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = db::get_pool().expect("Failed to get DB pool");

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .service(login)
                    .service(register)
                    .service(refresh),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}