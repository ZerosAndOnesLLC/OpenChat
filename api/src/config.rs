use std::env;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub tv_api_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub host: String,
    pub enable_tls: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub enable_rate_limiting: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let enable_tls = env::var("ENABLE_TLS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let tls_cert_path = env::var("TLS_CERT_PATH")
            .ok()
            .filter(|s| !s.is_empty());

        let tls_key_path = env::var("TLS_KEY_PATH")
            .ok()
            .filter(|s| !s.is_empty());

        let enable_rate_limiting = env::var("ENABLE_RATE_LIMITING")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            tv_api_url: env::var("TV_API_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            enable_tls,
            tls_cert_path,
            tls_key_path,
            enable_rate_limiting,
        })
    }
}
