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
