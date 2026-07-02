use crate::error::BlogClientError;
use crate::types::BlogApi;
use async_trait::async_trait;
use blog_proto::blog::blog_service_client::BlogServiceClient;
use blog_proto::blog::{
    CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest, ListPostsResponse,
    LoginRequest, PostResponse, RegisterRequest, UpdatePostRequest, User,
};
use tonic::metadata::MetadataValue;
use tonic::Request;

pub struct GrpcClient {
    client: BlogServiceClient<tonic::transport::Channel>,
    token: Option<String>,
}

impl GrpcClient {
    pub async fn connect(addr: String) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(addr).await?;
        Ok(Self { client, token: None })
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_string());
    }
    
    pub fn get_token(&self) -> String {
        self.token.clone().unwrap_or(String::new())
    }

    fn authed<T>(&self, inner: T) -> Result<Request<T>, BlogClientError> {
        let mut req = Request::new(inner);
        if let Some(token) = &self.token {
            let val: MetadataValue<_> = format!("Bearer {}", token)
                .parse()
                .map_err(|_| BlogClientError::Internal("invalid token format".to_string()))?;
            req.metadata_mut().insert("authorization", val);
        }
        Ok(req)
    }
}

#[async_trait]
impl BlogApi for GrpcClient {
    async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<User, BlogClientError> {
        let res = self.client.register(RegisterRequest {
            username: username.into(),
            email: email.into(),
            password: password.into(),
        }).await.map_err(BlogClientError::from)?.into_inner();

        self.token = Some(res.token);
        res.user.ok_or_else(|| BlogClientError::Internal("missing user in response".to_string()))
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<User, BlogClientError> {
        let res = self.client.login(LoginRequest {
            username: username.into(),
            password: password.into(),
        }).await.map_err(BlogClientError::from)?.into_inner();

        self.token = Some(res.token);
        res.user.ok_or_else(|| BlogClientError::Internal("missing user in response".to_string()))
    }

    async fn create_post(&mut self, title: &str, content: &str) -> Result<PostResponse, BlogClientError> {
        let req = self.authed(CreatePostRequest {
            title: title.into(),
            content: content.into(),
        })?;
        Ok(self.client.create_post(req).await.map_err(BlogClientError::from)?.into_inner())
    }

    async fn get_post(&mut self, id: i64) -> Result<Option<PostResponse>, BlogClientError> {
        match self.client.get_post(GetPostRequest { id }).await {
            Ok(res) => Ok(Some(res.into_inner())),
            Err(s) if s.code() == tonic::Code::NotFound => Ok(None),
            Err(s) => Err(BlogClientError::from(s)),
        }
    }

    async fn update_post(&mut self, id: i64, title: &str, content: &str) -> Result<PostResponse, BlogClientError> {
        let req = self.authed(UpdatePostRequest {
            id,
            title: title.into(),
            content: content.into(),
        })?;
        Ok(self.client.update_post(req).await.map_err(BlogClientError::from)?.into_inner())
    }

    async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let req = self.authed(DeletePostRequest { id })?;
        self.client.delete_post(req).await.map_err(BlogClientError::from)?;
        Ok(())
    }

    async fn list_posts(&mut self, limit: Option<u32>, offset: Option<u32>) -> Result<ListPostsResponse, BlogClientError> {
        let res = self.client.list_posts(ListPostsRequest {
            page: offset.unwrap_or(0),
            limit: limit.unwrap_or(10),
        }).await.map_err(BlogClientError::from)?;
        Ok(res.into_inner())
    }
}
