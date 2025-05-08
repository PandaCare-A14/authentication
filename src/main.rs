use std::net::{Ipv4Addr, SocketAddrV4};

use actix_web::{web, App, HttpServer};

mod db;
mod errors;
mod handlers;
mod models;
mod repository;
mod schema;
mod services;
mod tests;

use crate::handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::middleware::Logger;
    use env_logger;
    use log;

    let pool = db::get_pool().unwrap();

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

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
    .bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8000))?
    .run()
    .await
}
