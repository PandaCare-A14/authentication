use crate::{
    db::Connection,
    models::{
        jwt::{RefreshTokenCreationDTO, RefreshTokenDTO},
        users::User,
    },
};
use chrono::{Duration, Utc};
use diesel::{
    expression_methods::ExpressionMethods, insert_into, query_dsl::methods::FilterDsl, result,
    update, QueryResult, RunQueryDsl,
};

pub fn create_refresh_token(conn: &mut Connection, user: User, token: &str) -> QueryResult<String> {
    use crate::schema::refresh_tokens::dsl::*;

    loop {
        let new_token = RefreshTokenCreationDTO {
            token_str: token.to_string(),
            user_id: user.id.to_string(),
            expired_at: Utc::now().naive_utc() + Duration::minutes(30),
        };

        let created_token = insert_into(refresh_tokens)
            .values(new_token)
            .returning(token_str)
            .get_result(conn);

        match created_token {
            Ok(rt) => return Ok(rt),
            Err(result::Error::DatabaseError(result::DatabaseErrorKind::UniqueViolation, _)) => {
                continue
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn get_refresh_token(conn: &mut Connection, token: &str) -> QueryResult<Vec<RefreshTokenDTO>> {
    use crate::schema::refresh_tokens::dsl::*;

    let refresh_token = refresh_tokens
        .filter(token_str.eq(token))
        .get_results::<RefreshTokenDTO>(conn)?;

    Ok(refresh_token)
}

// Used to revoke a refresh token, returning the result of the update query
pub fn revoke_refresh_token(conn: &mut Connection, token: &str) -> QueryResult<()> {
    use crate::schema::refresh_tokens::dsl::*;

    update(refresh_tokens)
        .filter(token_str.eq(token))
        .set(is_revoked.eq(true))
        .execute(conn)
        .map(|_| Ok(()))?
}
