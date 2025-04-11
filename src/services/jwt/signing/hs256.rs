use jsonwebtoken::{encode, EncodingKey, Header};
use std::fs;

use crate::errors::jwt::JWTCreationError;

use super::TokenSigner;

pub struct HS256Signer {
    secret_key: EncodingKey,
}

impl HS256Signer {
    pub fn new(path: &str) -> Result<Self, JWTCreationError> {
        let key_file = fs::read(path).map_err(|_err| JWTCreationError::PrivateKeyNotFound)?;
        let secret_key = EncodingKey::from_secret(&key_file);
        Ok(Self { secret_key })
    }
}

impl TokenSigner for HS256Signer {
    fn sign(&self, claims: impl serde::Serialize) -> Result<String, JWTCreationError> {
        encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &self.secret_key,
        )
        .map_err(|_err| JWTCreationError::TokenEncodingFailure)
    }
}
