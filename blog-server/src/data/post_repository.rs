use crate::data::pg_repository::PgRepository;
use crate::domain::base::Pagination;
use crate::domain::error::PostError;
use crate::domain::post::Post;
use async_trait::async_trait;

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: Post) -> Result<Post, PostError>;
    async fn upsert(&self, post: Post) -> Result<Post, PostError>;
    async fn delete(&self, id: i64, user_id: i64) -> Result<bool, PostError>;
    async fn get_post_by_id(&self, id: i64) -> Result<Option<Post>, PostError>;
    async fn get_posts(&self, pagination: &Pagination) -> Result<Vec<Post>, PostError>;

    async fn get_posts_count(&self) -> Result<i64, PostError>;
}

#[async_trait]
impl PostRepository for PgRepository {
    async fn create(&self, post: Post) -> Result<Post, PostError> {
        Ok(sqlx::query_as!(
            Post,
            "INSERT INTO posts (title, content, author_id) VALUES ($1, $2, $3) RETURNING *",
            post.title,
            post.content,
            post.author_id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    async fn upsert(&self, post: Post) -> Result<Post, PostError> {
        Ok(sqlx::query_as!(
            Post,
            "UPDATE posts SET title = $1, content = $2, updated_at = now() WHERE id = $3 AND author_id = $4 RETURNING *",
            post.title,
            post.content,
            post.id,
            post.author_id,
        ).fetch_one(&self.pool).await?)
    }

    async fn delete(&self, id: i64, user_id: i64) -> Result<bool, PostError> {
        Ok(sqlx::query!(
            "DELETE FROM posts WHERE id = $1 AND author_id = $2",
            id,
            user_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }

    async fn get_post_by_id(&self, id: i64) -> Result<Option<Post>, PostError> {
        Ok(
            sqlx::query_as!(Post, "SELECT * FROM posts WHERE id = $1", id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    async fn get_posts(&self, pagination: &Pagination) -> Result<Vec<Post>, PostError> {
        Ok(sqlx::query_as!(
            Post,
            "SELECT * FROM posts ORDER BY id DESC LIMIT $1 OFFSET $2",
            pagination.limit.0,
            pagination.offset,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn get_posts_count(&self) -> Result<i64, PostError> {
        match sqlx::query!("SELECT COUNT(*) as count FROM posts")
            .fetch_one(&self.pool)
            .await?
            .count
        {
            Some(count) => Ok(count),
            None => Ok(0),
        }
    }
}
