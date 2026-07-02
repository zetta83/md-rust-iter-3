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

#[cfg(test)]
mod tests {
    use super::*;

    // --- JWT ---

    #[test]
    fn jwt_generate_and_verify_round_trip() {
        let keys = JwtKeys::new("test-secret".to_string());
        let token = keys.generate_token("42").unwrap();
        let claims = keys.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "42");
    }

    #[test]
    fn jwt_verify_rejects_wrong_secret() {
        let keys = JwtKeys::new("secret-a".to_string());
        let token = keys.generate_token("1").unwrap();

        let other = JwtKeys::new("secret-b".to_string());
        assert!(other.verify_token(&token).is_err());
    }

    #[test]
    fn jwt_verify_rejects_garbage() {
        let keys = JwtKeys::new("secret".to_string());
        assert!(keys.verify_token("not.a.token").is_err());
    }

    #[test]
    fn jwt_claims_contain_sub() {
        let keys = JwtKeys::new("s".to_string());
        let token = keys.generate_token("user-99").unwrap();
        let claims = keys.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user-99");
        assert!(claims.exp > claims.iat);
    }

    // --- Argon2 ---

    #[test]
    fn password_hash_and_verify_correct() {
        let hash = hash_password("correcthorse").unwrap();
        assert!(verify_password("correcthorse", &hash).unwrap());
    }

    #[test]
    fn password_verify_wrong_password() {
        let hash = hash_password("correcthorse").unwrap();
        assert!(!verify_password("wronghorse", &hash).unwrap());
    }

    #[test]
    fn password_hash_format_is_argon2() {
        let hash = hash_password("anypassword").unwrap();
        assert!(hash.starts_with("$argon2"));
    }
}
