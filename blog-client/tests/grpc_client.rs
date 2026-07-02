use blog_client::{BlogApi, BlogClient, BlogClientError};
use blog_proto::blog::blog_service_server::BlogServiceServer;
use blog_server::application::auth_service::AuthService;
use blog_server::application::blog_service::BlogService;
use blog_server::data::in_memory::{InMemoryPostRepository, InMemoryUserRepository};
use blog_server::data::post_repository::PostRepository;
use blog_server::data::user_repository::UserRepository;
use blog_server::infrastructure::jwt::JwtKeys;
use blog_server::presentation::grpc_service::BlogServiceImpl;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

const JWT_SECRET: &str = "test-secret";

async fn start_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let keys = JwtKeys::new(JWT_SECRET.into());

    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
    let post_repo: Arc<dyn PostRepository> = Arc::new(InMemoryPostRepository::new());
    let auth_srv = AuthService::new(Arc::clone(&user_repo), keys);
    let blog_srv = BlogService::new(Arc::clone(&post_repo));

    tokio::spawn(async move {
        Server::builder()
            .add_service(BlogServiceServer::new(BlogServiceImpl::new(auth_srv, blog_srv)))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    format!("http://{}", addr)
}

#[tokio::test]
async fn grpc_full_crud_flow() {
    let addr = start_server().await;
    let mut client = BlogClient::grpc(addr).await.unwrap();

    let user = client.register("alice", "alice@example.com", "password123").await.unwrap();
    assert_eq!(user.username, "alice");
    assert!(user.id > 0);

    let post = client.create_post("Hello", "World").await.unwrap();
    assert_eq!(post.title, "Hello");
    assert!(post.id > 0);

    let found = client.get_post(post.id).await.unwrap();
    assert_eq!(found.map(|p| p.id), Some(post.id));

    let list = client.list_posts(Some(10), Some(0)).await.unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.posts.len(), 1);

    let updated = client.update_post(post.id, "New Title", "New Content").await.unwrap();
    assert_eq!(updated.title, "New Title");

    client.delete_post(post.id).await.unwrap();
    assert!(client.get_post(post.id).await.unwrap().is_none());
}

#[tokio::test]
async fn grpc_login_then_post() {
    let addr = start_server().await;
    let mut setup = BlogClient::grpc(addr.clone()).await.unwrap();
    setup.register("bob", "bob@example.com", "bobpassword").await.unwrap();

    let mut client = BlogClient::grpc(addr).await.unwrap();
    client.login("bob", "bobpassword").await.unwrap();
    let post = client.create_post("Via Login", "content").await.unwrap();
    assert_eq!(post.title, "Via Login");
}

#[tokio::test]
async fn grpc_create_post_without_token_is_unauthorized() {
    let addr = start_server().await;
    let mut client = BlogClient::grpc(addr).await.unwrap();

    let err = client.create_post("Title", "Content").await.unwrap_err();
    assert!(matches!(err, BlogClientError::Unauthorized));
}

#[tokio::test]
async fn grpc_get_nonexistent_post_returns_none() {
    let addr = start_server().await;
    let mut client = BlogClient::grpc(addr).await.unwrap();

    assert!(client.get_post(99999).await.unwrap().is_none());
}

#[tokio::test]
async fn grpc_delete_others_post_is_not_found() {
    let addr = start_server().await;

    let mut author = BlogClient::grpc(addr.clone()).await.unwrap();
    author.register("author", "author@example.com", "authorpass").await.unwrap();
    let post = author.create_post("Mine", "content").await.unwrap();

    let mut other = BlogClient::grpc(addr).await.unwrap();
    other.register("other", "other@example.com", "otherpass").await.unwrap();
    let err = other.delete_post(post.id).await.unwrap_err();
    assert!(matches!(err, BlogClientError::NotFound(_)));
}
