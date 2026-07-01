use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::data::pg_repository::PgRepository;
use crate::data::post_repository::PostRepository;
use crate::data::user_repository::UserRepository;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::database::{create_pool, run_migrations};
use crate::infrastructure::jwt::JwtKeys;
use crate::infrastructure::logging::init_logging;
use crate::presentation::grpc_service::BlogServiceImpl;
use actix_web::{App, HttpServer, middleware::Logger, web};
use std::net::SocketAddr;
use std::sync::Arc;

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env().expect("Failed to read configuration from environment");
    init_logging();

    let pg_pool = create_pool(config.database_url.as_str())
        .await
        .expect("Failed to create database pool");
    run_migrations(&pg_pool)
        .await
        .expect("Failed to run migrations");

    let jwt_keys = JwtKeys::new(config.jwt_secret.clone());
    let pg_repo = Arc::new(PgRepository::new(pg_pool));
    let auth_srv = AuthService::new(
        Arc::clone(&pg_repo) as Arc<dyn UserRepository>,
        jwt_keys.clone(),
    );
    let blog_srv = BlogService::new(Arc::clone(&pg_repo) as Arc<dyn PostRepository>);

    tokio::select! {
        result = run_rest_server(&config, auth_srv.clone(), blog_srv.clone()) => result,
        result = run_grpc_server(&config, auth_srv, blog_srv) => result,
    }
}

async fn run_grpc_server(
    config: &AppConfig,
    auth_srv: AuthService<dyn UserRepository>,
    blog_srv: BlogService<dyn PostRepository>,
) -> std::io::Result<()> {
    use blog_proto::blog::blog_service_server::BlogServiceServer;
    use tonic::transport::Server;

    let addr = SocketAddr::new(
        config
            .grpc_host
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?,
        config.grpc_port,
    );

    Server::builder()
        .add_service(BlogServiceServer::new(BlogServiceImpl::new(
            auth_srv, blog_srv,
        )))
        .serve(addr)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

async fn run_rest_server(
    config: &AppConfig,
    auth_srv: AuthService<dyn UserRepository>,
    blog_srv: BlogService<dyn PostRepository>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(auth_srv.clone()))
            .app_data(web::Data::new(blog_srv.clone()))
            .configure(presentation::http_handlers::configure(auth_srv.keys()))
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
