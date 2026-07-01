use crate::application::auth_service::AuthService;
use crate::application::blog_service;
use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::domain;
use crate::domain::base::{DeserializeString, Email, Password};
use crate::domain::error::{PostError, UserError};
use blog_proto::blog::blog_service_server::BlogService;
use blog_proto::blog::{
    AuthResponse, CreatePostRequest, DeletePostRequest, DeletePostResponse, GetPostRequest,
    ListPostsRequest, ListPostsResponse, LoginRequest, PostResponse, RegisterRequest,
    UpdatePostRequest, User,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct BlogServiceImpl {
    auth_service: Arc<AuthService<dyn UserRepository>>,
    blog_service: Arc<blog_service::BlogService<dyn PostRepository>>,
}

impl BlogServiceImpl {
    pub fn new(
        auth_service: AuthService<dyn UserRepository>,
        blog_service: blog_service::BlogService<dyn PostRepository>,
    ) -> Self {
        Self {
            auth_service: Arc::new(auth_service),
            blog_service: Arc::new(blog_service),
        }
    }

    fn extract_user_id<T>(&self, request: &Request<T>) -> Result<i64, Status> {
        let header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("missing authorization metadata"))?;

        let value = header
            .to_str()
            .map_err(|_| Status::unauthenticated("invalid authorization header"))?;

        let token = value
            .strip_prefix("Bearer ")
            .ok_or_else(|| Status::unauthenticated("invalid authorization header format"))?;

        let claims = self
            .auth_service
            .keys()
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        claims
            .sub
            .parse::<i64>()
            .map_err(|_| Status::unauthenticated("invalid token claims"))
    }

    fn build_auth_response(
        &self,
        user: domain::user::User,
    ) -> Result<Response<AuthResponse>, Status> {
        Ok(Response::new(AuthResponse {
            token: self
                .auth_service
                .generate_token(&user)
                .map_err(|e| Status::internal(format!("failed generate token: {}", e)))?,
            user: Some(user.into()),
        }))
    }
}

#[tonic::async_trait]
impl BlogService for BlogServiceImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        if req.username.is_empty() || req.password.is_empty() || req.email.is_empty() {
            return Err(Status::invalid_argument(
                "invalid email, username or password",
            ));
        }

        let email = Email::new(req.email).map_err(|_| Status::invalid_argument("invalid email"))?;
        let hashed_password = Password::new(req.password)
            .map_err(|_| Status::invalid_argument("invalid password"))?;

        self.build_auth_response(
            self.auth_service
                .register(req.username, email, hashed_password)
                .await
                .map_err(|e| match e {
                    UserError::Internal(msg) => Status::internal(msg),
                    _ => Status::invalid_argument("registration failed"),
                })?,
        )
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        if req.username.is_empty() || req.password.is_empty() {
            return Err(Status::invalid_argument("invalid username or password"));
        }

        self.build_auth_response(
            self.auth_service
                .login(req.username, req.password)
                .await
                .map_err(|e| match e {
                    UserError::Internal(msg) => Status::internal(msg),
                    _ => Status::invalid_argument("login failed"),
                })?,
        )
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();

        let limit = if req.limit == 0 { 10 } else { req.limit };

        let posts = self
            .blog_service
            .get_list_posts(req.page, limit)
            .await
            .map_err(|e| Status::internal(format!("failed to get list posts: {}", e)))?;

        Ok(Response::new(ListPostsResponse {
            posts: posts.into_iter().map(PostResponse::from).collect(),
            // todo: в проде лучше использовать один запрос для списка и кол-ва, для учебного проекта сойдет и так
            total: self
                .blog_service
                .get_posts_count()
                .await
                .map_err(|e| Status::internal(format!("failed to get posts count: {}", e)))?,
            offset: req.page,
            limit,
        }))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        Ok(Response::new(PostResponse::from(
            self.blog_service
                .get_post_by_id(request.into_inner().id)
                .await
                .map_err(|e| match e {
                    PostError::NotFound => Status::not_found("post not found"),
                    PostError::Internal(msg) => Status::internal(msg),
                    _ => Status::internal("unexpected error"),
                })?,
        )))
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.extract_user_id(&request)?;
        let req = request.into_inner();

        if req.title.is_empty() || req.content.is_empty() {
            return Err(Status::invalid_argument("title and content are required"));
        }

        Ok(Response::new(PostResponse::from(
            self.blog_service
                .create_post(&req.title, &req.content, user_id)
                .await
                .map_err(|e| match e {
                    PostError::Internal(msg) => Status::internal(msg),
                    _ => Status::internal("unexpected error"),
                })?,
        )))
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.extract_user_id(&request)?;
        let req = request.into_inner();

        if req.id < 1 || req.title.is_empty() || req.content.is_empty() {
            return Err(Status::invalid_argument(
                "id, title and content are required",
            ));
        }

        Ok(Response::new(PostResponse::from(
            self.blog_service
                .update_post(req.id, user_id, &req.title, &req.content)
                .await
                .map_err(|e| match e {
                    PostError::NotFound => Status::not_found("post not found"),
                    PostError::Internal(msg) => Status::internal(msg),
                    _ => Status::internal("unexpected error"),
                })?,
        )))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let user_id = self.extract_user_id(&request)?;
        let req = request.into_inner();

        self.blog_service
            .delete_post(req.id, user_id)
            .await
            .map_err(|e| match e {
                PostError::NotFound => Status::not_found("post not found"),
                PostError::Internal(msg) => Status::internal(msg),
                _ => Status::internal("unexpected error"),
            })?;

        Ok(Response::new(DeletePostResponse {}))
    }
}

impl From<domain::user::User> for User {
    fn from(u: domain::user::User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
        }
    }
}

impl From<domain::post::Post> for PostResponse {
    fn from(p: domain::post::Post) -> Self {
        PostResponse {
            id: p.id,
            title: p.title,
            content: p.content,
            author_id: p.author_id,
        }
    }
}
