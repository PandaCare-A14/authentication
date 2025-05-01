use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct RefreshTokenCreationDTO {
    pub token_str: String,
    pub user_id: String,
    pub expired_at: NaiveDateTime,
}

#[derive(Queryable)]
pub struct RefreshTokenDTO {
    pub token: String,
    pub user_id: String,
    pub expired_at: NaiveDateTime,
    pub revoked: bool,
}
