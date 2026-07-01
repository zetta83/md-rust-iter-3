use std::future::{ready, Ready};
use crate::data::user_repository::UserRepository;
use crate::domain::base::{Email, Password};
use crate::domain::error::UserError;
use crate::domain::user::User;
use crate::infrastructure::jwt::{JwtKeys, verify_password};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use actix_web::error::ErrorUnauthorized;
use std::sync::Arc;
use tracing::instrument;

pub struct AuthService<R: UserRepository + 'static + ?Sized> {
    repo: Arc<R>,
    keys: JwtKeys,
}

impl<R: UserRepository + ?Sized + 'static> Clone for AuthService<R> {
    fn clone(&self) -> Self {
        Self {
            repo: Arc::clone(&self.repo),
            keys: self.keys.clone(),
        }
    }
}

impl<R: ?Sized + UserRepository + 'static> AuthService<R> {
    pub fn new(repo: Arc<R>, keys: JwtKeys) -> Self {
        Self { repo, keys }
    }

    pub fn keys(&self) -> &JwtKeys {
        &self.keys
    }

    #[instrument(skip(self))]
    pub async fn register(
        &self,
        username: String,
        email: Email,
        password: Password,
    ) -> Result<User, UserError> {
        self.repo
            .create(User::new(username, email, password.0))
            .await
            .map_err(UserError::from)
    }

    #[instrument(skip(self))]
    pub async fn login(&self, username: String, password: String) -> Result<User, UserError> {
        let user = self
            .repo
            .find_by_username(username.as_str())
            .await
            .map_err(|_| UserError::InvalidCredentials)?
            .ok_or_else(|| UserError::InvalidCredentials)?;
        if verify_password(password.as_str(), &user.password_hash)
            .map_err(|_| UserError::InvalidCredentials)?
        {
            Ok(user)
        } else {
            Err(UserError::InvalidCredentials)
        }
    }

    #[instrument(skip(self))]
    pub fn generate_token(&self, user: &User) -> Result<String, UserError> {
        self.keys
            .generate_token(user.id.to_string().as_str())
            .map_err(|e| UserError::Internal(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn get_user(&self, id: i64) -> Result<User, UserError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(UserError::from)?
            .ok_or_else(|| UserError::NotFound(id.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: i64,
    #[allow(dead_code)]
    pub email: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            return ready(Ok(user.clone()));
        }
        ready(Err(ErrorUnauthorized("missing authenticated user")))
    }
}

pub async fn extract_user_from_token(
    token: &str,
    keys: &JwtKeys,
    auth_service: &AuthService<dyn UserRepository>,
) -> Result<AuthenticatedUser, Error> {
    let claims = keys
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| ErrorUnauthorized("invalid token"))?;
    let user = auth_service
        .get_user(user_id)
        .await
        .map_err(|_| ErrorUnauthorized("user not found"))?;

    Ok(AuthenticatedUser {
        id: user.id,
        email: user.email,
    })
}
