use anyhow::anyhow;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|e| anyhow!("failed to parse port: {}", e))?;
        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL must be set"))?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| anyhow!("JWT_SECRET must be set"))?;

        let cors_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            host,
            port,
            database_url,
            jwt_secret,
            cors_origins,
        })
    }
}
