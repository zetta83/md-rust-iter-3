use actix_web::{web, App, HttpServer};
use blog_client::{BlogApi, BlogClient, BlogClientError};
use blog_server::application::auth_service::AuthService;
use blog_server::application::blog_service::BlogService;
use blog_server::data::in_memory::{InMemoryPostRepository, InMemoryUserRepository};
use blog_server::data::post_repository::PostRepository;
use blog_server::data::user_repository::UserRepository;
use blog_server::infrastructure::jwt::JwtKeys;
use blog_server::presentation::http_handlers;
use std::sync::Arc;

const JWT_SECRET: &str = "test-secret";

struct TestServer {
    base_url: String,
    handle: actix_web::dev::ServerHandle,
}

async fn start_server() -> TestServer {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let keys = JwtKeys::new(JWT_SECRET.into());

    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
    let post_repo: Arc<dyn PostRepository> = Arc::new(InMemoryPostRepository::new());
    let auth_srv = AuthService::new(Arc::clone(&user_repo), keys.clone());
    let blog_srv = BlogService::new(Arc::clone(&post_repo));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth_srv.clone()))
            .app_data(web::Data::new(blog_srv.clone()))
            .configure(http_handlers::configure(auth_srv.keys()))
    })
    .listen(listener)
    .unwrap()
    .run();

    let handle = server.handle();
    tokio::spawn(server);

    TestServer {
        base_url: format!("http://127.0.0.1:{}/api", port),
        handle,
    }
}

#[tokio::test]
async fn http_full_crud_flow() {
    let srv = start_server().await;
    let mut client = BlogClient::http(srv.base_url.clone());

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

    srv.handle.stop(false).await;
}

#[tokio::test]
async fn http_login_then_post() {
    let srv = start_server().await;
    let mut setup = BlogClient::http(srv.base_url.clone());
    setup.register("bob", "bob@example.com", "bobpassword").await.unwrap();

    let mut client = BlogClient::http(srv.base_url.clone());
    client.login("bob", "bobpassword").await.unwrap();
    let post = client.create_post("Via Login", "content").await.unwrap();
    assert_eq!(post.title, "Via Login");

    srv.handle.stop(false).await;
}

#[tokio::test]
async fn http_create_post_without_token_is_unauthorized() {
    let srv = start_server().await;
    let mut client = BlogClient::http(srv.base_url.clone());

    let err = client.create_post("Title", "Content").await.unwrap_err();
    assert!(matches!(err, BlogClientError::Unauthorized));

    srv.handle.stop(false).await;
}

#[tokio::test]
async fn http_get_nonexistent_post_returns_none() {
    let srv = start_server().await;
    let mut client = BlogClient::http(srv.base_url.clone());

    assert!(client.get_post(99999).await.unwrap().is_none());

    srv.handle.stop(false).await;
}

#[tokio::test]
async fn http_delete_others_post_is_not_found() {
    let srv = start_server().await;

    let mut author = BlogClient::http(srv.base_url.clone());
    author.register("author", "author@example.com", "authorpass").await.unwrap();
    let post = author.create_post("Mine", "content").await.unwrap();

    let mut other = BlogClient::http(srv.base_url.clone());
    other.register("other", "other@example.com", "otherpass").await.unwrap();
    let err = other.delete_post(post.id).await.unwrap_err();
    assert!(matches!(err, BlogClientError::NotFound(_)));

    srv.handle.stop(false).await;
}
