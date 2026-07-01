use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct JwtKeys {
    secret: String,
}

impl JwtKeys {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(
        &self,
        user_id: &str,
    ) -> anyhow::Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: chrono::Utc::now()
                .checked_add_signed(chrono::Duration::hours(1))
                .unwrap()
                .timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> anyhow::Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    // Пример: ~19 MiB памяти, 2 итерации, 1 параллель — подбирайте под свою цель (50–150 мс)
    let params = Params::new(19 * 1024, 2, 1, None)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
