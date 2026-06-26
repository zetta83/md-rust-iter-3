use crate::domain::base::{Email, DeserializeString};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    username: String,
    #[serde(deserialize_with = "Email::validate")]
    email: Email,
    password_hash: String,
    created_at: DateTime<Utc>,
}

impl User {}
