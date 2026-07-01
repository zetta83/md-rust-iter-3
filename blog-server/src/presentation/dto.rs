use crate::domain::base::DeserializeString;
use crate::domain::base::{Email, Password};
use crate::domain::post::Post;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    #[serde(deserialize_with = "Email::validate")]
    pub email: Email,
    #[serde(deserialize_with = "Password::validate")]
    pub password: Password,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PostCreateRequest {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ListPostsResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
        }
    }
}
