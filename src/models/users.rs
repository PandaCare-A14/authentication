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
    pub role: Role,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{},{}}}", self.id, self.email)
    }
}

#[derive(Clone, Debug, Deserialize, diesel_derive_enum::DbEnum)]
#[serde(field_identifier, rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::Role"]
pub enum Role {
    Pacilian,
    Caregiver,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let snake_case = match *self {
            Self::Pacilian => "pacilian",
            Self::Caregiver => "caregiver",
        };

        write!(f, "{}", snake_case)
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let snake_case = match *self {
            Self::Pacilian => "pacilian",
            Self::Caregiver => "caregiver",
        };

        serializer.serialize_str(snake_case)
    }
}

#[derive(Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct InsertableUser {
    pub email: String,
    pub password: String,
    pub role: Role,
}

#[derive(Deserialize)]
pub struct LoginFields {
    pub email: String,
    pub password: String,
}
