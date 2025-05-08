use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub name: String,
    pub nik: BigDecimal,
    pub phone_number: BigDecimal,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{},{},{},{}}}",
            self.email, self.name, self.nik, self.phone_number
        )
    }
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct InsertableUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub nik: BigDecimal,
    pub phone_number: BigDecimal,
}

#[derive(Deserialize)]
pub struct LoginFields {
    pub email: String,
    pub password: String,
}
