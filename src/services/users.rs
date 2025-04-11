use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{rand_core::OsRng, SaltString};

use crate::{
    db::Connection,
    errors::users::{UserCreationError, UserValidationError},
    models::users::{InsertableUser, LoginFields, User},
    repository::users::{get_user_by_email, insert_new_user},
};

pub fn create_user(
    conn: &mut Connection,
    new_user: InsertableUser,
) -> Result<User, UserCreationError> {
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

    insert_new_user(conn, final_user).map_err(|_err| UserCreationError::UserInsertionError)
}

pub fn validate_user(
    conn: &mut Connection,
    user_credentials: LoginFields,
) -> Result<User, UserValidationError> {
    let user = get_user_by_email(conn, &user_credentials.email)
        .map_err(|_err| UserValidationError::InvalidEmail)?;

    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|_err| UserValidationError::InvalidPasswordFormat)?;

    let _ = Argon2::default()
        .verify_password(&user_credentials.password.as_bytes(), &parsed_hash)
        .map_err(|_err| UserValidationError::IncorrectPassword);

    Ok(user)
}
