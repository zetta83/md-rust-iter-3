use crate::application::auth_service::AuthService;
use crate::domain::base::{DeserializeString, Email, Password};
use crate::domain::error::UserError;
use crate::infrastructure::jwt::JwtKeys;
use super::in_memory::InMemoryUserRepository;
use std::sync::Arc;

fn make_service() -> AuthService<InMemoryUserRepository> {
    AuthService::new(
        Arc::new(InMemoryUserRepository::new()),
        JwtKeys::new("test-secret".into()),
    )
}

fn email(s: &str) -> Email {
    Email::new(s.to_string()).unwrap()
}

fn password(s: &str) -> Password {
    Password::new(s.to_string()).unwrap()
}

#[tokio::test]
async fn register_creates_user_with_id() {
    let svc = make_service();
    let user = svc
        .register("alice".into(), email("alice@example.com"), password("password123"))
        .await
        .unwrap();
    assert_eq!(user.username, "alice");
    assert_eq!(user.email, "alice@example.com");
    assert!(user.id > 0);
}

#[tokio::test]
async fn register_duplicate_email_returns_already_exists() {
    let svc = make_service();
    svc.register("alice".into(), email("same@example.com"), password("password123"))
        .await
        .unwrap();
    let err = svc
        .register("alice2".into(), email("same@example.com"), password("password456"))
        .await
        .unwrap_err();
    assert!(matches!(err, UserError::AlreadyExists));
}

#[tokio::test]
async fn login_correct_credentials_returns_user() {
    let svc = make_service();
    svc.register("bob".into(), email("bob@example.com"), password("secret123"))
        .await
        .unwrap();
    let user = svc.login("bob".into(), "secret123".into()).await.unwrap();
    assert_eq!(user.username, "bob");
}

#[tokio::test]
async fn login_wrong_password_returns_invalid_credentials() {
    let svc = make_service();
    svc.register("carol".into(), email("carol@example.com"), password("correct_pass"))
        .await
        .unwrap();
    let err = svc
        .login("carol".into(), "wrong_pass".into())
        .await
        .unwrap_err();
    assert!(matches!(err, UserError::InvalidCredentials));
}

#[tokio::test]
async fn login_unknown_user_returns_invalid_credentials() {
    let svc = make_service();
    let err = svc.login("nobody".into(), "pass".into()).await.unwrap_err();
    assert!(matches!(err, UserError::InvalidCredentials));
}

#[tokio::test]
async fn generate_token_encodes_user_id() {
    let svc = make_service();
    let user = svc
        .register("dave".into(), email("dave@example.com"), password("davepass1"))
        .await
        .unwrap();
    let token = svc.generate_token(&user).unwrap();
    let claims = svc.keys().verify_token(&token).unwrap();
    assert_eq!(claims.sub, user.id.to_string());
}

#[tokio::test]
async fn get_user_returns_registered_user() {
    let svc = make_service();
    let created = svc
        .register("eve".into(), email("eve@example.com"), password("evepass12"))
        .await
        .unwrap();
    let found = svc.get_user(created.id).await.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.username, "eve");
}

#[tokio::test]
async fn get_user_unknown_id_returns_not_found() {
    let svc = make_service();
    let err = svc.get_user(999).await.unwrap_err();
    assert!(matches!(err, UserError::NotFound(_)));
}
