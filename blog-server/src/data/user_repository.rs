use crate::data::pg_repository::PgRepository;
use crate::domain::error::UserError;
use crate::domain::user::User;
use async_trait::async_trait;
use tracing::error;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, UserError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, UserError>;

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError>;
}

#[async_trait]
impl UserRepository for PgRepository {
    async fn create(&self, user: User) -> Result<User, UserError> {
        let res = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
            user.username,
            user.email,
            user.password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to create user: {}", e);
            if e.as_database_error()
                .and_then(|db| db.constraint())
                .map(|c| c.contains("users_email"))
                == Some(true)
            {
                UserError::AlreadyExists
            } else {
                UserError::Internal(format!("database error: {}", e))
            }
        })?;

        Ok(res)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, UserError> {
        let res = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to find user by id {}: {}", id, e);
                UserError::Internal(format!("database error: {}", e))
            })?;
        Ok(Some(res))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError> {
        let res = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to find user by username {}: {}", username, e);
                UserError::Internal(format!("database error: {}", e))
            })?;
        Ok(Some(res))
    }
}
