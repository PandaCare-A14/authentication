use std::{fs, io};

use jsonwebtoken::{encode, jwk::JwkSet, EncodingKey, Header};

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
        let jwks_str: &str = &fs::read_to_string("jwks/jwks.json")
            .map_err(|err| JWTCreationError::InvalidTokenData(err.to_string()))?;

        // Assume we only have one Key
        let jwks: JwkSet = serde_json::from_str::<JwkSet>(jwks_str)
            .map_err(|err| JWTCreationError::InvalidTokenData(err.to_string()))?;

        let jwk = jwks.keys.first().ok_or(JWTCreationError::InvalidTokenData(
            "JWK not found".to_string(),
        ))?;

        let kid = jwk
            .common
            .key_id
            .clone()
            .ok_or(JWTCreationError::InvalidTokenData(
                "JWK doesn't have a key ID".to_string(),
            ))?;

        encode(
            &Header {
                typ: Some("JWT".to_string()),
                alg: jsonwebtoken::Algorithm::RS256,
                kid: Some(kid),
                ..Default::default()
            },
            &claims,
            &self.secret_key,
        )
        .map_err(|_err| JWTCreationError::TokenEncodingFailure)
    }
}
