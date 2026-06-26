use crate::domain::error::DomainError;
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
