use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserCreationError {
    #[error("Password hashing failed")]
    PasswordHashError,
    #[error("User insertion failed")]
    UserInsertionError,
}

#[derive(Debug, Error)]
pub enum UserValidationError {
    #[error("Password is incorrect")]
    IncorrectPassword,
    #[error("Email doesn't exist")]
    InvalidEmail,
    #[error("A database integrity error has occurred. Please contact site administrator")]
    InvalidPasswordFormat,
    #[error("User not found")]
    UserNotFound,
}
