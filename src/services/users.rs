use crate::{
    db::Connection,
    errors::UserCreationError,
    interfaces::users::{InsertableUser, User},
    repository::users::insert_new_user,
};

pub fn create_user(
    conn: &mut Connection,
    new_user: InsertableUser,
) -> Result<User, UserCreationError> {
    insert_new_user(conn, new_user)
}
