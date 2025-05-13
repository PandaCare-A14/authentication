use jsonwebtoken::{encode, EncodingKey, Header};

use crate::errors::jwt::JWTCreationError;

use super::TokenSigner;

pub struct HS256Signer {
    secret_key: EncodingKey,
}

impl HS256Signer {
    pub fn new(base64_key: &str) -> Result<Self, JWTCreationError> {
        let secret_key = EncodingKey::from_base64_secret(base64_key.trim())
            .map_err(|_err| JWTCreationError::InvalidPrivateKey)?;
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
