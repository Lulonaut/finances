use crate::result::ErrorType;
use actix_web::dev::Payload;
use actix_web::http::header::AUTHORIZATION;
use actix_web::{FromRequest, HttpRequest};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::Deserialize;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn hash_password(password: &String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_hash(hash: &str, password: &String) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash);
    if parsed_hash.is_err() {
        return false;
    }
    argon2
        .verify_password(password.as_bytes(), &parsed_hash.unwrap())
        .is_ok()
}

#[derive(Deserialize, Serialize)]
struct Claims {
    uid: i64,
    exp: u64,
}

pub fn create_jwt(uid: i64) -> Result<String, anyhow::Error> {
    let jwt_secret = std::env::var("JWT_SECRET")?;

    let exp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + (60 * 60 * 24 * 365);
    let claims = Claims { uid, exp };

    let header = jsonwebtoken::Header::new(Algorithm::HS512);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn decode_jwt(jwt: &str) -> Option<i64> {
    let jwt_secret = std::env::var("JWT_SECRET");
    if jwt_secret.is_err() {
        return None;
    }

    let decoded = jsonwebtoken::decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(jwt_secret.unwrap().as_bytes()),
        &Validation::new(Algorithm::HS512),
    );
    if decoded.is_err() {
        return None;
    }

    Some(decoded.unwrap().claims.uid)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub uid: i64,
}

impl FromRequest for UserClaims {
    type Error = crate::result::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get(AUTHORIZATION);
        if auth_header.is_none() {
            return std::future::ready(Err(Self::Error {
                cause: ErrorType::BadRequest,
            }));
        }
        let value = auth_header.unwrap().to_str().unwrap();
        if !value.starts_with("Bearer ") {
            return std::future::ready(Err(Self::Error {
                cause: ErrorType::BadRequest,
            }));
        }

        // Remove: "Bearer "
        let token = &value[7..];
        match decode_jwt(token) {
            None => std::future::ready(Err(Self::Error {
                cause: ErrorType::BadRequest,
            })),
            Some(claim) => std::future::ready(Ok(UserClaims { uid: claim })),
        }
    }
}
