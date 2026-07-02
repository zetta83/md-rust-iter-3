use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
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

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ResponseError for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DomainError::Forbidden(_) => StatusCode::FORBIDDEN,
            DomainError::NotFound(_) => StatusCode::NOT_FOUND,
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            DomainError::AlreadyExists(_) => StatusCode::BAD_REQUEST,
            DomainError::InvalidCredentials => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            DomainError::Forbidden(_) => None,
            DomainError::NotFound(_) => None,
            DomainError::Validation(_) => None,
            DomainError::Internal(_) => None,
            DomainError::AlreadyExists(_) => None,
            DomainError::InvalidCredentials => None,
        };

        let payload = ErrorEnvelope {
            error: &message,
            details,
        };

        HttpResponse::build(self.status_code()).json(payload)
    }
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user not found with id {0}")]
    NotFound(String),
    #[error("user already exists")]
    AlreadyExists,
    #[error("server error {0}")]
    Internal(String),
    #[error("invalid credentials")]
    InvalidCredentials,
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
            sqlx::Error::RowNotFound => UserError::NotFound("not found".into()),
            e => UserError::Internal(e.to_string()),
        }
    }
}

impl From<UserError> for DomainError {
    fn from(err: UserError) -> Self {
        match err {
            UserError::NotFound(_) => DomainError::NotFound("user".into()),
            UserError::AlreadyExists => DomainError::AlreadyExists("user".into()),
            UserError::Internal(e) => DomainError::Internal(e),
            UserError::InvalidCredentials => DomainError::InvalidCredentials,
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::ResponseError;
    use actix_web::http::StatusCode;

    // --- DomainError → HTTP status ---

    #[test]
    fn domain_error_status_codes() {
        assert_eq!(DomainError::Internal("x".into()).status_code(),     StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(DomainError::Forbidden("x".into()).status_code(),    StatusCode::FORBIDDEN);
        assert_eq!(DomainError::NotFound("x".into()).status_code(),     StatusCode::NOT_FOUND);
        assert_eq!(DomainError::Validation("x".into()).status_code(),   StatusCode::BAD_REQUEST);
        assert_eq!(DomainError::AlreadyExists("x".into()).status_code(),StatusCode::BAD_REQUEST);
        assert_eq!(DomainError::InvalidCredentials.status_code(),       StatusCode::UNAUTHORIZED);
    }

    // --- From<sqlx::Error> ---

    #[test]
    fn sqlx_row_not_found_becomes_domain_not_found() {
        let err: DomainError = sqlx::Error::RowNotFound.into();
        assert!(matches!(err, DomainError::NotFound(_)));
    }

    #[test]
    fn sqlx_other_becomes_domain_internal() {
        let err: DomainError = sqlx::Error::Protocol("db issue".into()).into();
        assert!(matches!(err, DomainError::Internal(_)));
    }

    #[test]
    fn sqlx_row_not_found_becomes_post_not_found() {
        let err: PostError = sqlx::Error::RowNotFound.into();
        assert!(matches!(err, PostError::NotFound));
    }

    #[test]
    fn sqlx_other_becomes_post_internal() {
        let err: PostError = sqlx::Error::Protocol("db issue".into()).into();
        assert!(matches!(err, PostError::Internal(_)));
    }

    #[test]
    fn sqlx_row_not_found_becomes_user_not_found() {
        let err: UserError = sqlx::Error::RowNotFound.into();
        assert!(matches!(err, UserError::NotFound(_)));
    }

    #[test]
    fn sqlx_other_becomes_user_internal() {
        let err: UserError = sqlx::Error::Protocol("db issue".into()).into();
        assert!(matches!(err, UserError::Internal(_)));
    }

    // --- From<UserError> for DomainError ---

    #[test]
    fn user_not_found_becomes_domain_not_found() {
        let err: DomainError = UserError::NotFound("42".into()).into();
        assert!(matches!(err, DomainError::NotFound(_)));
    }

    #[test]
    fn user_already_exists_becomes_domain_already_exists() {
        let err: DomainError = UserError::AlreadyExists.into();
        assert!(matches!(err, DomainError::AlreadyExists(_)));
    }

    #[test]
    fn user_invalid_credentials_becomes_domain_invalid_credentials() {
        let err: DomainError = UserError::InvalidCredentials.into();
        assert!(matches!(err, DomainError::InvalidCredentials));
    }

    #[test]
    fn user_internal_becomes_domain_internal() {
        let err: DomainError = UserError::Internal("boom".into()).into();
        assert!(matches!(err, DomainError::Internal(_)));
    }

    // --- From<PostError> for DomainError ---

    #[test]
    fn post_not_found_becomes_domain_not_found() {
        let err: DomainError = PostError::NotFound.into();
        assert!(matches!(err, DomainError::NotFound(_)));
    }

    #[test]
    fn post_forbidden_becomes_domain_forbidden() {
        let err: DomainError = PostError::Forbidden.into();
        assert!(matches!(err, DomainError::Forbidden(_)));
    }

    #[test]
    fn post_internal_becomes_domain_internal() {
        let err: DomainError = PostError::Internal("boom".into()).into();
        assert!(matches!(err, DomainError::Internal(_)));
    }
}
