use crate::domain::base::DeserializeString;
use crate::domain::base::{Email, Password};
use crate::domain::post::Post;
use serde::Deserialize;

pub use blog_dto::{AuthResponse, ListPostsResponse, PostResponse, User};

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
