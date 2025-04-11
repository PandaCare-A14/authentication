use jsonwebtoken::{encode, EncodingKey, Header};
use std::fs;

use crate::errors::jwt::JWTCreationError;

use super::TokenSigner;

pub struct ES256Signer {
    signing_key: EncodingKey,
}

impl ES256Signer {
    pub fn new(path: &str) -> Result<Self, JWTCreationError> {
        let key_file = fs::read(path).map_err(|_err| JWTCreationError::PrivateKeyNotFound)?;
        let signing_key = EncodingKey::from_ec_pem(&key_file)
            .map_err(|_err| JWTCreationError::InvalidPrivateKey)?;
        Ok(Self { signing_key })
    }
}

impl TokenSigner for ES256Signer {
    fn sign(&self, claims: impl serde::Serialize) -> Result<String, JWTCreationError> {
        encode(
            &Header::new(jsonwebtoken::Algorithm::ES256),
            &claims,
            &self.signing_key,
        )
        .map_err(|_err| JWTCreationError::TokenEncodingFailure)
    }
}
