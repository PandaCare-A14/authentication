use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    db::Connection,
    models::users::{InsertableUser, User},
};

pub fn get_user_by_email(conn: &mut Connection, user_email: &str) -> QueryResult<User> {
    use crate::schema::users::dsl::*;

    let user = users.filter(email.eq(user_email)).first::<User>(conn)?;

    Ok(user)
}

pub fn get_user_by_id(conn: &mut Connection, user_id: Uuid) -> QueryResult<User> {
    use crate::schema::users::dsl::*;

    let user = users.filter(id.eq(user_id)).first::<User>(conn)?;

    Ok(user)
}

pub fn insert_new_user(conn: &mut Connection, new_user: InsertableUser) -> QueryResult<User> {
    use crate::schema::users::dsl::*;

    let user = diesel::insert_into(users)
        .values(new_user)
        .get_result::<User>(conn)?;

    Ok(user)
}
