use crate::error::BlogClientError;
use crate::types::BlogApi;
use async_trait::async_trait;
use blog_proto::blog::{AuthResponse, ListPostsResponse, PostResponse, User};
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct ErrorJson {
    error: String,
}

pub struct HttpClient {
    base_url: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            token: None,
            client: reqwest::Client::builder()
                .user_agent("blog-client/1.0")
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(15))
                .build()
                .expect("failed to build reqwest client"),
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_string());
    }

    pub fn get_token(&self) -> String {
        self.token.clone().unwrap_or_default()
    }

    async fn check_status(res: reqwest::Response) -> Result<reqwest::Response, BlogClientError> {
        let status = res.status();
        if status.is_success() {
            return Ok(res);
        }
        let msg = res
            .json::<ErrorJson>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| status.to_string());
        Err(match status {
            StatusCode::UNAUTHORIZED => BlogClientError::Unauthorized,
            StatusCode::FORBIDDEN => BlogClientError::PermissionDenied,
            StatusCode::NOT_FOUND => BlogClientError::NotFound(msg),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                BlogClientError::InvalidRequest(msg)
            }
            _ => BlogClientError::Internal(msg),
        })
    }

    fn bearer(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }
}

#[async_trait]
impl BlogApi for HttpClient {
    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, BlogClientError> {
        let res = self
            .client
            .post(format!("{}/auth/register", self.base_url))
            .json(
                &serde_json::json!({ "username": username, "email": email, "password": password }),
            )
            .send()
            .await?;

        let auth: AuthResponse = Self::check_status(res).await?.json().await?;
        self.token = Some(auth.token);
        auth.user
            .ok_or_else(|| BlogClientError::Internal("missing user in response".to_string()))
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<User, BlogClientError> {
        let res = self
            .client
            .post(format!("{}/auth/login", self.base_url))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;

        let auth: AuthResponse = Self::check_status(res).await?.json().await?;
        self.token = Some(auth.token);
        auth.user
            .ok_or_else(|| BlogClientError::Internal("missing user in response".to_string()))
    }

    async fn list_posts(
        &mut self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<ListPostsResponse, BlogClientError> {
        let res = self
            .client
            .get(format!(
                "{}/posts?page={}&limit={}",
                self.base_url,
                offset.unwrap_or(0),
                limit.unwrap_or(10)
            ))
            .send()
            .await?;

        Ok(Self::check_status(res).await?.json().await?)
    }

    async fn get_post(&mut self, id: i64) -> Result<Option<PostResponse>, BlogClientError> {
        let res = self
            .client
            .get(format!("{}/posts/{}", self.base_url, id))
            .send()
            .await?;

        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(Self::check_status(res).await?.json().await?))
    }

    async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        let mut req = self
            .client
            .post(format!("{}/posts", self.base_url))
            .json(&serde_json::json!({ "title": title, "content": content }));

        if let Some(auth) = self.bearer() {
            req = req.header(reqwest::header::AUTHORIZATION, auth);
        }

        Ok(Self::check_status(req.send().await?).await?.json().await?)
    }

    async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        let mut req = self
            .client
            .put(format!("{}/posts/{}", self.base_url, id))
            .json(&serde_json::json!({ "title": title, "content": content }));

        if let Some(auth) = self.bearer() {
            req = req.header(reqwest::header::AUTHORIZATION, auth);
        }

        Ok(Self::check_status(req.send().await?).await?.json().await?)
    }

    async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let mut req = self
            .client
            .delete(format!("{}/posts/{}", self.base_url, id));

        if let Some(auth) = self.bearer() {
            req = req.header(reqwest::header::AUTHORIZATION, auth);
        }

        Self::check_status(req.send().await?).await?;
        Ok(())
    }
}
