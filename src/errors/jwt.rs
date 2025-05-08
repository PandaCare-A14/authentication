use thiserror::Error;

use super::users::UserValidationError;

#[derive(Debug, Error)]
pub enum JWTError {
    #[error("Failed to create JWT: {0}")]
    JWTCreation(#[from] self::JWTCreationError),

    #[error("Failed to validate JWT: {0}")]
    JWTValidation(#[from] self::JWTValidationError),

    #[error("User validation failed: {0}")]
    UserValidation(#[from] UserValidationError),
}

#[derive(Debug, Error)]
pub enum JWTValidationError {
    #[error("Token is invalid")]
    TokenInvalid,
    #[error("Refresh token has been revoked")]
    TokenRevoked,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Failed to fetch refresh token")]
    TokenFetchingFailure,
    #[error("A duplicate refresh token was found. Please contact the site administrator")]
    DuplicateToken,
    #[error("Refresh token was not found on the database")]
    TokenNotFound,
}

#[derive(Debug, Error)]
pub enum JWTCreationError {
    #[error("Private key file was not found")]
    PrivateKeyNotFound,
    #[error("An error occurred while encoding the JWT")]
    TokenEncodingFailure,
    #[error("Invalid signing key")]
    InvalidPrivateKey,
    #[error("Refresh token creation failed")]
    RefreshTokenGenerationFailure,
}
