use crate::errors::jwt::JWTCreationError;

pub mod es256;
pub mod hs256;
pub mod rs256;

pub trait TokenSigner {
    fn sign(&self, claims: impl serde::Serialize) -> Result<String, JWTCreationError>;
}
