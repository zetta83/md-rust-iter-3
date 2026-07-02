use crate::domain::error::DomainError;
use crate::infrastructure::jwt::hash_password;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

pub trait DeserializeString
where
    Self: Sized,
{
    fn new(value: String) -> Result<Self, DomainError>;

    fn validate<'de, D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(String::deserialize(d)?).map_err(D::Error::custom)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email(pub String);

impl DeserializeString for Email {
    fn new(email: String) -> Result<Self, DomainError> {
        if email.len() < 6 {
            return Err(DomainError::Validation("email too short".to_string()));
        }
        if !email.contains('@') {
            return Err(DomainError::Validation("invalid email format".to_string()));
        }

        Ok(Self(email.to_lowercase()))
    }
}

impl Into<Email> for String {
    fn into(self) -> Email {
        Email(self)
    }
}

impl Into<String> for Email {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password(pub String);

impl DeserializeString for Password {
    fn new(password: String) -> Result<Self, DomainError> {
        if password.len() < 8 {
            return Err(DomainError::Validation(
                "password must be at least 8 characters".to_string(),
            ));
        }

        Ok(Self(
            hash_password(password.as_str())
                .map_err(|e| DomainError::Internal(e.to_string()))?
                .to_string(),
        ))
    }
}

impl Into<Password> for String {
    fn into(self) -> Password {
        Password(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: i64,
    #[serde(default)]
    pub limit: PaginationLimit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginationLimit(pub i64);
impl Default for PaginationLimit {
    fn default() -> Self {
        PaginationLimit(10)
    }
}

impl Into<PaginationLimit> for i64 {
    fn into(self) -> PaginationLimit {
        PaginationLimit(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Email ---

    #[test]
    fn email_valid() {
        assert!(Email::new("user@example.com".to_string()).is_ok());
    }

    #[test]
    fn email_lowercased_on_create() {
        let email = Email::new("User@Example.COM".to_string()).unwrap();
        assert_eq!(email.0, "user@example.com");
    }

    #[test]
    fn email_too_short() {
        // "a@b.c" — 5 символов, меньше минимума
        let err = Email::new("a@b.c".to_string()).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn email_missing_at() {
        let err = Email::new("noemail.com".to_string()).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn email_boundary_exactly_6_chars() {
        // "a@b.cd" — 6 символов, должен пройти
        assert!(Email::new("a@b.cd".to_string()).is_ok());
    }

    // --- Password ---

    #[test]
    fn password_too_short() {
        let err = Password::new("short".to_string()).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn password_exactly_8_chars() {
        assert!(Password::new("12345678".to_string()).is_ok());
    }

    #[test]
    fn password_is_hashed_on_create() {
        let pwd = Password::new("mysecretpass".to_string()).unwrap();
        // хранится Argon2id-хеш, а не сам пароль
        assert!(pwd.0.starts_with("$argon2"));
        assert_ne!(pwd.0, "mysecretpass");
    }

    #[test]
    fn password_two_hashes_differ() {
        // Argon2id использует случайную соль, поэтому хеши одного пароля не совпадают
        let a = Password::new("samepassword".to_string()).unwrap();
        let b = Password::new("samepassword".to_string()).unwrap();
        assert_ne!(a.0, b.0);
    }
}
