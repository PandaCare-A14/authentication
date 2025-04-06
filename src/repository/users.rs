use log::debug;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::prelude::*;
use password_hash::{rand_core::OsRng, SaltString};

use crate::{
    db::Connection,
    errors::{UserCreationError, UserValidationError},
    interfaces::users::{InsertableUser, LoginFields, User},
    schema::users,
};

pub fn validate_user(
    conn: &mut Connection,
    user_credentials: LoginFields,
) -> Result<User, UserValidationError> {
    use crate::schema::users::dsl::*;

    let user_email = &user_credentials.email;

    let user = users
        .filter(email.eq(user_email))
        .first::<User>(conn)
        .map_err(|_err| UserValidationError::InvalidEmail)?;

    let parsed_hash = PasswordHash::new(&user.password).unwrap();

    let _ = Argon2::default()
        .verify_password(&user_credentials.password.as_bytes(), &parsed_hash)
        .map_err(|_err| UserValidationError::IncorrectPassword);

    Ok(user)
}

pub fn insert_new_user(
    conn: &mut Connection,
    new_user: InsertableUser,
) -> Result<User, UserCreationError> {
    // hash and salt pw
    let password: &String = &new_user.password;
    let salt: SaltString = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash: String = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_err| UserCreationError::PasswordHashError)?
        .to_string();

    let final_user = InsertableUser {
        password: password_hash,
        ..new_user
    };

    // normal diesel operations
    let user = diesel::insert_into(users::table)
        .values(final_user)
        .get_result::<User>(conn)
        .map_err(|_err| UserCreationError::UserInsertionError)?;

    debug!("{}", &user);

    Ok(user)
}
