use crate::domain::base::{Email};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub(crate) id: i64,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
    pub(crate) created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: Email, password_hash: String) -> Self {
        Self {
            id: 0,
            username,
            email: email.into(),
            password_hash,
            created_at: Utc::now(),
        }
    }
}
