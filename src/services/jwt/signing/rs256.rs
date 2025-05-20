use jsonwebtoken::{encode, EncodingKey, Header};

use crate::errors::jwt::JWTCreationError;

use super::TokenSigner;

pub struct RS256Signer {
    secret_key: EncodingKey,
}

impl RS256Signer {
    pub fn new(secret_key: &str) -> Result<Self, JWTCreationError> {
        let secret_key = EncodingKey::from_rsa_pem(secret_key.as_bytes())
            .map_err(|_err| JWTCreationError::InvalidPrivateKey)?;
        Ok(Self { secret_key })
    }
}

impl TokenSigner for RS256Signer {
    fn sign(&self, claims: impl serde::Serialize) -> Result<String, JWTCreationError> {
        encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &self.secret_key,
        )
        .map_err(|_err| JWTCreationError::TokenEncodingFailure)
    }
}
