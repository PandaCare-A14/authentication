use crate::{
    db::Connection,
    errors::jwt::{JWTCreationError, JWTError, JWTValidationError},
    models::{jwt::RefreshTokenDTO, users::User},
    repository::{
        jwt::{create_refresh_token, get_refresh_token, revoke_refresh_token},
        users::get_user_by_id,
    },
};

use chrono::{DateTime, Duration, Utc};
use rand::{distr::Alphanumeric, rng, Rng};
use serde::{Deserialize, Serialize};
use signing::{hs256::HS256Signer, rs256::RS256Signer, TokenSigner};
use uuid::{self, Uuid};

mod signing;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredClaims {
    pub iss: String, // Issuer
    // pub sub: String, // Subject
    // pub aud: String, // Audience
    pub exp: usize,  // Expiration time (as UTC timestamp)
    pub nbf: usize,  // Not before (as UTC timestamp)
    pub iat: usize,  // Issued at (as UTC timestamp)
    pub jti: String, // JWT ID (unique identifier for the token)
}

impl RegisteredClaims {
    pub fn new(iss: &str, seconds_to_expiration: i64) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let exp: DateTime<Utc> = now + Duration::seconds(seconds_to_expiration);

        RegisteredClaims {
            iss: iss.to_string(),
            exp: exp.timestamp() as usize,
            nbf: now.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    #[serde(flatten)]
    pub registered_claims: RegisteredClaims,
    pub user_id: String,
    pub roles: Vec<String>,
}

#[derive(Serialize)]
pub struct Jwt {
    pub access: String,
    pub refresh: String,
}

#[derive(Deserialize)]
pub struct RefreshInfo {
    pub refresh_token: String,
}

pub type RevocationInfo = RefreshInfo;

pub fn generate_jwt(
    conn: &mut Connection,
    secret_key: String,
    user: User,
) -> Result<Jwt, JWTCreationError> {
    let registered_claims = RegisteredClaims::new("Pandacare", 300);

    let claims = Claims {
        registered_claims,
        user_id: user.id.to_string(),
        roles: vec![user.role.to_string()],
    };

    let signer: RS256Signer = RS256Signer::new(&secret_key)?;

    let access_token = signer
        .sign(claims)
        .map_err(|_err| JWTCreationError::TokenEncodingFailure)?;

    let random_str: String = rng()
        .sample_iter(Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    let refresh_token = create_refresh_token(conn, user, &random_str)
        .map_err(|_err| JWTCreationError::RefreshTokenGenerationFailure)?;

    Ok(Jwt {
        access: access_token,
        refresh: refresh_token,
    })
}

pub fn refresh_token(
    conn: &mut Connection,
    secret_key: String,
    token_str: &str,
) -> Result<Jwt, JWTError> {
    use crate::errors::users::UserValidationError;

    let refresh_token: Vec<RefreshTokenDTO> = get_refresh_token(conn, token_str)
        .map_err(|_err| JWTError::JWTValidation(JWTValidationError::TokenFetchingFailure))?;

    let refresh_token: &RefreshTokenDTO = match refresh_token.as_slice() {
        [only] => only,
        [] => return Err(JWTError::JWTValidation(JWTValidationError::TokenNotFound)),
        _ => return Err(JWTError::JWTValidation(JWTValidationError::DuplicateToken)),
    };

    let user_id = Uuid::parse_str(&refresh_token.user_id)
        .map_err(|_err| JWTError::JWTValidation(JWTValidationError::TokenInvalid))?;

    let user = get_user_by_id(conn, user_id)
        .map_err(|_err| JWTError::UserValidation(UserValidationError::UserNotFound))?;

    if refresh_token.revoked {
        Err(JWTError::JWTValidation(JWTValidationError::TokenRevoked))
    } else if Utc::now().naive_utc() > refresh_token.expired_at {
        Err(JWTError::JWTValidation(JWTValidationError::TokenExpired))
    } else {
        revoke_refresh_token(conn, &refresh_token.token)
            .map_err(|_err| JWTError::JWTValidation(JWTValidationError::TokenNotFound))?;

        let jwt = generate_jwt(conn, secret_key, user).map_err(JWTError::JWTCreation)?;
        Ok(jwt)
    }
}
