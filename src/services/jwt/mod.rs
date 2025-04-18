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
use signing::{hs256::HS256Signer, TokenSigner};
use uuid::{self, Uuid};

mod signing;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredClaims {
    pub iss: String, // Issuer
    // pub sub: String, // Subject
    pub aud: String, // Audience
    pub exp: usize,  // Expiration time (as UTC timestamp)
    pub nbf: usize,  // Not before (as UTC timestamp)
    pub iat: usize,  // Issued at (as UTC timestamp)
    pub jti: String, // JWT ID (unique identifier for the token)
}

impl RegisteredClaims {
    pub fn new(iss: &str, aud: &str, seconds_to_expiration: i64) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let exp: DateTime<Utc> = now + Duration::seconds(seconds_to_expiration);

        RegisteredClaims {
            iss: iss.to_string(),
            aud: aud.to_string(),
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
    pub role: String,
}

#[derive(Serialize)]
pub struct JWT {
    pub access: String,
    pub refresh: String,
}

#[derive(Deserialize)]
pub struct RefreshInfo {
    pub refresh_token: String,
}

pub fn generate_jwt(conn: &mut Connection, user: User) -> Result<JWT, JWTCreationError> {
    // TODO: Modularize claims config
    let registered_claims = RegisteredClaims::new("Pandacare", "https://www.pandacare.com", 300);

    let claims = Claims {
        registered_claims,
        user_id: user.id.to_string(),
        role: "pacilian".to_string(),
    };

    let signer: HS256Signer = HS256Signer::new("private-key.pem")?;

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

    Ok(JWT {
        access: access_token,
        refresh: refresh_token,
    })
}

pub fn refresh_token(conn: &mut Connection, token_str: &str) -> Result<JWT, JWTError> {
    use crate::errors::users::UserValidationError;

    let refresh_token: Vec<RefreshTokenDTO> = get_refresh_token(conn, token_str)
        .map_err(|_err| JWTError::JWTValidationError(JWTValidationError::TokenFetchingFailure))?;

    let refresh_token = match refresh_token.as_slice() {
        [only] => only,
        [] => {
            return Err(JWTError::JWTValidationError(
                JWTValidationError::TokenNotFound,
            ))
        }
        _ => {
            return Err(JWTError::JWTValidationError(
                JWTValidationError::DuplicateToken,
            ))
        }
    };

    let user_id = Uuid::parse_str(&refresh_token.user_id)
        .map_err(|_err| JWTError::JWTValidationError(JWTValidationError::TokenInvalid))?;

    let user = get_user_by_id(conn, user_id)
        .map_err(|_err| JWTError::UserValidationError(UserValidationError::UserNotFound))?;

    if refresh_token.revoked {
        Err(JWTError::JWTValidationError(
            JWTValidationError::TokenRevoked,
        ))
    } else if Utc::now().naive_utc() > refresh_token.expired_at {
        Err(JWTError::JWTValidationError(
            JWTValidationError::TokenExpired,
        ))
    } else {
        revoke_refresh_token(conn, &refresh_token.token)
            .map_err(|_err| JWTError::JWTValidationError(JWTValidationError::TokenNotFound))?;

        let jwt = generate_jwt(conn, user).map_err(|err| JWTError::JWTCreationError(err))?;
        Ok(jwt)
    }
}
