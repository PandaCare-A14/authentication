use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;
use std::env;
use std::error::Error;

pub type Connection = PooledConnection<ConnectionManager<PgConnection>>;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn get_pool() -> Result<DbPool, Box<dyn Error>> {
    let connection_manager = ConnectionManager::<PgConnection>::new(env::var("DATABASE_URL")?);

    let pool = r2d2::Pool::builder().build(connection_manager)?;

    Ok(pool)
}
