use crate::data::post_repository::PostRepository;
use crate::domain::base::Pagination;
use crate::domain::error::PostError;
use crate::domain::post::Post;
use std::sync::Arc;
use tracing::instrument;

pub struct BlogService<R: PostRepository + 'static + ?Sized> {
    repo: Arc<R>,
}

impl<R: PostRepository + ?Sized + 'static> Clone for BlogService<R> {
    fn clone(&self) -> Self {
        Self {
            repo: Arc::clone(&self.repo),
        }
    }
}

impl<R: PostRepository + 'static + ?Sized> BlogService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, PostError> {
        Ok(self
            .repo
            .create(Post::new(title.to_string(), content.to_string(), author_id))
            .await
            .map_err(PostError::from)?)
    }

    #[instrument(skip(self))]
    pub async fn get_list_posts(&self, page: u32, limit: u32) -> Result<Vec<Post>, PostError> {
        Ok(self
            .repo
            .get_posts(&Pagination {
                offset: page.into(),
                limit: (limit as i64).into(),
            })
            .await
            .map_err(PostError::from)?)
    }

    #[instrument(skip(self))]
    pub async fn get_posts_count(&self) -> Result<i64, PostError> {
        Ok(self.repo.get_posts_count().await.map_err(PostError::from)?)
    }

    #[instrument(skip(self))]
    pub async fn get_post_by_id(&self, id: i64) -> Result<Post, PostError> {
        match self
            .repo
            .get_post_by_id(id)
            .await
            .map_err(PostError::from)?
        {
            Some(post) => Ok(post),
            None => Err(PostError::NotFound),
        }
    }

    #[instrument(skip(self))]
    pub async fn update_post(
        &self,
        id: i64,
        user_id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, PostError> {
        let mut new_post = Post::new(title.to_string(), content.to_string(), user_id);
        new_post.id = id;

        Ok(self.repo.upsert(new_post).await.map_err(PostError::from)?)
    }

    #[instrument(skip(self))]
    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), PostError> {
        match self.repo.delete(id, user_id).await.map_err(PostError::from)? {
            true => Ok(()),
            false => Err(PostError::NotFound),
        }
    }
}
