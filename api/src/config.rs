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
    pub encryption_secret: String,
    pub websocket: WebSocketConfig,
    pub redis: RedisConfig,
    /// Base URL for the API (used for generating webhook URLs)
    pub api_base_url: String,
    /// Base URL for the Web UI (used for desktop app redirects)
    pub webui_url: String,
    /// LiveKit configuration for voice/video calling (optional)
    pub livekit: Option<LiveKitConfig>,
}

#[derive(Debug, Clone)]
pub struct LiveKitConfig {
    pub url: String,
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RedisConfig {
    /// Enable Redis Cluster mode
    pub enable_cluster: bool,
    /// Redis Cluster nodes (comma-separated URLs)
    pub cluster_nodes: Vec<String>,
    /// Enable cache warming on connection
    pub enable_cache_warming: bool,
    /// Cache TTL for channel data (seconds)
    pub channel_cache_ttl: u64,
    /// Cache TTL for messages (seconds)
    pub message_cache_ttl: u64,
    /// Cache TTL for user presence (seconds)
    pub presence_cache_ttl: u64,
    /// Enable Redis pub/sub for cross-server events
    pub enable_pubsub: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
        let redis = RedisConfig::from_env();

        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a valid number");

        // Default API base URL based on host/port, can be overridden for production
        let api_base_url = env::var("API_BASE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{}", port));

        // Web UI URL for desktop app redirects
        let webui_url = env::var("WEBUI_URL")
            .unwrap_or_else(|_| "https://openchat.zerosandones.us".to_string());

        let livekit = LiveKitConfig::from_env();

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            tv_api_url: env::var("TV_API_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            port,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            enable_tls,
            tls_cert_path,
            tls_key_path,
            enable_rate_limiting,
            encryption_secret: env::var("ENCRYPTION_SECRET")
                .unwrap_or_else(|_| env::var("JWT_SECRET").unwrap_or_else(|_| "default-fallback-secret".to_string())),
            websocket,
            redis,
            api_base_url,
            webui_url,
            livekit,
        })
    }
}

impl RedisConfig {
    pub fn from_env() -> Self {
        let enable_cluster = env::var("REDIS_ENABLE_CLUSTER")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let cluster_nodes = if enable_cluster {
            env::var("REDIS_CLUSTER_NODES")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        };

        Self {
            enable_cluster,
            cluster_nodes,
            enable_cache_warming: env::var("REDIS_ENABLE_CACHE_WARMING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            channel_cache_ttl: env::var("REDIS_CHANNEL_CACHE_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            message_cache_ttl: env::var("REDIS_MESSAGE_CACHE_TTL")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap_or(120),
            presence_cache_ttl: env::var("REDIS_PRESENCE_CACHE_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            enable_pubsub: env::var("REDIS_ENABLE_PUBSUB")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
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

impl LiveKitConfig {
    pub fn from_env() -> Option<Self> {
        let url = env::var("LIVEKIT_URL").ok().filter(|s| !s.is_empty())?;
        let api_key = env::var("LIVEKIT_API_KEY").ok().filter(|s| !s.is_empty())?;
        let api_secret = env::var("LIVEKIT_API_SECRET").ok().filter(|s| !s.is_empty())?;
        Some(Self { url, api_key, api_secret })
    }
}
