use crate::error::BlogClientError;
use crate::grpc_client::GrpcClient;
use crate::http_client::HttpClient;
use async_trait::async_trait;
use blog_proto::blog::{ListPostsResponse, PostResponse, User};

#[async_trait]
pub trait BlogApi: Send + Sync {
    async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<User, BlogClientError>;
    async fn login(&mut self, username: &str, password: &str) -> Result<User, BlogClientError>;
    async fn create_post(&mut self, title: &str, content: &str) -> Result<PostResponse, BlogClientError>;
    async fn get_post(&mut self, id: i64) -> Result<Option<PostResponse>, BlogClientError>;
    async fn update_post(&mut self, id: i64, title: &str, content: &str) -> Result<PostResponse, BlogClientError>;
    async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError>;
    async fn list_posts(&mut self, limit: Option<u32>, offset: Option<u32>) -> Result<ListPostsResponse, BlogClientError>;
}

pub enum BlogClient {
    Http(HttpClient),
    Grpc(GrpcClient),
}

impl BlogClient {
    pub fn http(base_url: String) -> Self {
        BlogClient::Http(HttpClient::new(base_url))
    }

    pub async fn grpc(addr: String) -> Result<Self, BlogClientError> {
        Ok(BlogClient::Grpc(GrpcClient::connect(addr).await?))
    }

    pub fn set_token(&mut self, token: &str) {
        match self {
            BlogClient::Http(c) => c.set_token(token),
            BlogClient::Grpc(c) => c.set_token(token),
        }
    }
}

#[async_trait]
impl BlogApi for BlogClient {
    async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<User, BlogClientError> {
        match self {
            BlogClient::Http(c) => c.register(username, email, password).await,
            BlogClient::Grpc(c) => c.register(username, email, password).await,
        }
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<User, BlogClientError> {
        match self {
            BlogClient::Http(c) => c.login(username, password).await,
            BlogClient::Grpc(c) => c.login(username, password).await,
        }
    }

    async fn create_post(&mut self, title: &str, content: &str) -> Result<PostResponse, BlogClientError> {
        match self {
            BlogClient::Http(c) => c.create_post(title, content).await,
            BlogClient::Grpc(c) => c.create_post(title, content).await,
        }
    }

    async fn get_post(&mut self, id: i64) -> Result<Option<PostResponse>, BlogClientError> {
        match self {
            BlogClient::Http(c) => c.get_post(id).await,
            BlogClient::Grpc(c) => c.get_post(id).await,
        }
    }

    async fn update_post(&mut self, id: i64, title: &str, content: &str) -> Result<PostResponse, BlogClientError> {
        match self {
            BlogClient::Http(c) => c.update_post(id, title, content).await,
            BlogClient::Grpc(c) => c.update_post(id, title, content).await,
        }
    }

    async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        match self {
            BlogClient::Http(c) => c.delete_post(id).await,
            BlogClient::Grpc(c) => c.delete_post(id).await,
        }
    }

    async fn list_posts(&mut self, limit: Option<u32>, offset: Option<u32>) -> Result<ListPostsResponse, BlogClientError> {
        match self {
            BlogClient::Http(c) => c.list_posts(limit, offset).await,
            BlogClient::Grpc(c) => c.list_posts(limit, offset).await,
        }
    }
}
