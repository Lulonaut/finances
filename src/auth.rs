use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_hash(hash: String, password: String) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&hash);
    if parsed_hash.is_err() {
        return false;
    }
    argon2
        .verify_password(password.as_bytes(), &parsed_hash.unwrap())
        .is_ok()
}
