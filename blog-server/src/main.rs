use crate::infrastructure::config::AppConfig;
use crate::infrastructure::logging::init_logging;
use actix_web::{App, HttpServer, middleware::Logger};

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env().expect("Failed to read configuration from environment");
    init_logging();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(presentation::http_handlers::configure())
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
