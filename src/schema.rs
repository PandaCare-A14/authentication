// @generated automatically by Diesel CLI.

diesel::table! {
    refresh_tokens (token_str) {
        #[max_length = 255]
        token_str -> Varchar,
        #[max_length = 255]
        user_id -> Varchar,
        expired_at -> Timestamp,
        is_revoked -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        nik -> Numeric,
        phone_number -> Numeric,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    refresh_tokens,
    users,
);
