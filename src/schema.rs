// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role"))]
    pub struct Role;
}

diesel::table! {
    refresh_tokens (token_str) {
        #[max_length = 255]
        token_str -> Varchar,
        #[max_length = 255]
        user_id -> Varchar,
        expired_at -> Timestamp,
        is_revoked -> Bool,
        issued_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Role;

    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        role -> Role,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    refresh_tokens,
    users,
);
