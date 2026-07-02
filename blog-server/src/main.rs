use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use blog_server::application::auth_service::AuthService;
use blog_server::application::blog_service::BlogService;
use blog_server::data::pg_repository::PgRepository;
use blog_server::data::post_repository::PostRepository;
use blog_server::data::user_repository::UserRepository;
use blog_server::infrastructure::config::AppConfig;
use blog_server::infrastructure::database::{create_pool, run_migrations};
use blog_server::infrastructure::jwt::JwtKeys;
use blog_server::infrastructure::logging::init_logging;
use blog_server::presentation::grpc_service::BlogServiceImpl;
use blog_server::presentation::http_handlers;
use std::net::SocketAddr;
use std::sync::Arc;

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
        .map_err(std::io::Error::other)
}

async fn run_rest_server(
    config: &AppConfig,
    auth_srv: AuthService<dyn UserRepository>,
    blog_srv: BlogService<dyn PostRepository>,
) -> std::io::Result<()> {
    let config_data = config.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(build_cors(&config_data))
            .app_data(web::Data::new(auth_srv.clone()))
            .app_data(web::Data::new(blog_srv.clone()))
            .configure(http_handlers::configure(auth_srv.keys()))
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}

fn build_cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::AUTHORIZATION,
        ])
        .supports_credentials()
        .max_age(3600);

    for origin in &config.cors_origins {
        cors = cors.allowed_origin(origin.as_str());
    }

    cors
}
