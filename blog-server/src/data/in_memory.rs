use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::domain::base::Pagination;
use crate::domain::error::{PostError, UserError};
use crate::domain::post::Post;
use crate::domain::user::User;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Mutex;

pub struct InMemoryUserRepository {
    users: Mutex<Vec<User>>,
    next_id: Mutex<i64>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, mut user: User) -> Result<User, UserError> {
        let mut users = self.users.lock().unwrap();
        if users.iter().any(|u| u.email == user.email) {
            return Err(UserError::AlreadyExists);
        }
        let mut id = self.next_id.lock().unwrap();
        user.id = *id;
        *id += 1;
        users.push(user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, UserError> {
        Ok(self.users.lock().unwrap().iter().find(|u| u.id == id).cloned())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError> {
        Ok(self.users.lock().unwrap().iter().find(|u| u.username == username).cloned())
    }
}

pub struct InMemoryPostRepository {
    posts: Mutex<Vec<Post>>,
    next_id: Mutex<i64>,
}

impl InMemoryPostRepository {
    pub fn new() -> Self {
        Self {
            posts: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

#[async_trait]
impl PostRepository for InMemoryPostRepository {
    async fn create(&self, mut post: Post) -> Result<Post, PostError> {
        let mut posts = self.posts.lock().unwrap();
        let mut id = self.next_id.lock().unwrap();
        post.id = *id;
        *id += 1;
        posts.push(post.clone());
        Ok(post)
    }

    async fn upsert(&self, post: Post) -> Result<Post, PostError> {
        let mut posts = self.posts.lock().unwrap();
        let p = posts
            .iter_mut()
            .find(|p| p.id == post.id && p.author_id == post.author_id)
            .ok_or(PostError::NotFound)?;
        p.title = post.title.clone();
        p.content = post.content.clone();
        p.updated_at = Utc::now();
        Ok(p.clone())
    }

    async fn delete(&self, id: i64, user_id: i64) -> Result<bool, PostError> {
        let mut posts = self.posts.lock().unwrap();
        let before = posts.len();
        posts.retain(|p| !(p.id == id && p.author_id == user_id));
        Ok(posts.len() < before)
    }

    async fn get_post_by_id(&self, id: i64) -> Result<Option<Post>, PostError> {
        Ok(self.posts.lock().unwrap().iter().find(|p| p.id == id).cloned())
    }

    async fn get_posts(&self, pagination: &Pagination) -> Result<Vec<Post>, PostError> {
        let posts = self.posts.lock().unwrap();
        let offset = pagination.offset as usize;
        let limit = pagination.limit.0 as usize;
        let mut sorted: Vec<Post> = posts.clone();
        sorted.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(sorted.into_iter().skip(offset).take(limit).collect())
    }

    async fn get_posts_count(&self) -> Result<i64, PostError> {
        Ok(self.posts.lock().unwrap().len() as i64)
    }
}
