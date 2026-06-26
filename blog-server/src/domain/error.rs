use sqlx;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("server error {0}")]
    Internal(String),
    #[error("resource already exists: {0}")]
    AlreadyExists(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("resource forbidden: {0}")]
    Forbidden(String),
    #[error("resource not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user not found")]
    NotFound,
    #[error("user already exists")]
    AlreadyExists,
    #[error("server error {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum PostError {
    #[error("post not found")]
    NotFound,
    #[error("post forbidden")]
    Forbidden,
    #[error("server error {0}")]
    Internal(String),
}

impl From<sqlx::Error> for DomainError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            // todo: добавить какой именно ресурс не был найден
            sqlx::Error::RowNotFound => DomainError::NotFound(String::from("not found")),
            e => DomainError::Internal(e.to_string()),
        }
    }
}

impl From<sqlx::Error> for PostError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => PostError::NotFound,
            e => PostError::Internal(e.to_string()),
        }
    }
}

impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => UserError::NotFound,
            e => UserError::Internal(e.to_string()),
        }
    }
}

impl From<UserError> for DomainError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound => DomainError::NotFound("user".into()),
            UserError::AlreadyExists => DomainError::AlreadyExists("user".into()),
            UserError::Internal(e) => DomainError::Internal(e),
        }
    }
}

impl From<PostError> for DomainError {
    fn from(err: PostError) -> Self {
        match err {
            PostError::NotFound => DomainError::NotFound("post".into()),
            PostError::Forbidden => DomainError::Forbidden("post".into()),
            PostError::Internal(e) => DomainError::Internal(e),
        }
    }
}
