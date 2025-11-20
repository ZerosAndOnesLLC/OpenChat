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
    pub websocket: WebSocketConfig,
}

#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum number of concurrent WebSocket connections
    pub max_connections: usize,
    /// Maximum connections per user (to prevent abuse)
    pub max_connections_per_user: usize,
    /// Enable message batching
    pub enable_batching: bool,
    /// Batch size (number of messages)
    pub batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Enable compression for messages
    pub enable_compression: bool,
    /// Compression threshold in bytes (only compress messages larger than this)
    pub compression_threshold: usize,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Client timeout in seconds (no heartbeat response)
    pub client_timeout_secs: u64,
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

        let websocket = WebSocketConfig::from_env();

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
            websocket,
        })
    }
}

impl WebSocketConfig {
    pub fn from_env() -> Self {
        Self {
            max_connections: env::var("WS_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            max_connections_per_user: env::var("WS_MAX_CONNECTIONS_PER_USER")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            enable_batching: env::var("WS_ENABLE_BATCHING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            batch_size: env::var("WS_BATCH_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            batch_timeout_ms: env::var("WS_BATCH_TIMEOUT_MS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            enable_compression: env::var("WS_ENABLE_COMPRESSION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            compression_threshold: env::var("WS_COMPRESSION_THRESHOLD")
                .unwrap_or_else(|_| "1024".to_string())
                .parse()
                .unwrap_or(1024),
            heartbeat_interval_secs: env::var("WS_HEARTBEAT_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            client_timeout_secs: env::var("WS_CLIENT_TIMEOUT_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        }
    }
}
