//! Configuration management system for HackerExperience
//!
//! Provides centralized configuration management with environment-based overrides,
//! secret management, feature flags, and hot-reloading capabilities.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

/// Main configuration structure for HackerExperience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeConfig {
    /// Application settings
    pub app: AppConfig,
    /// Database configurations (13 databases)
    pub databases: DatabasesConfig,
    /// WebSocket server configuration
    pub websocket: WebSocketConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Game-specific settings
    pub game: GameConfig,
    /// Feature flags
    pub features: FeatureFlags,
    /// External services configuration
    pub services: ServicesConfig,
    /// Performance tuning
    pub performance: PerformanceConfig,
}

/// Application-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    /// Version
    pub version: String,
    /// Environment (development, staging, production)
    pub environment: Environment,
    /// Debug mode
    pub debug: bool,
    /// Base URL for the application
    pub base_url: String,
    /// HTTP server bind address
    pub bind_address: String,
    /// HTTP server port
    pub port: u16,
    /// Number of worker threads
    pub workers: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Maximum request size in bytes
    pub max_request_size: u64,
}

/// Environment types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// Database configurations for all 13 HackerExperience databases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasesConfig {
    /// Main game database
    pub main: DatabaseConfig,
    /// Cache database
    pub cache: DatabaseConfig,
    /// Logging database
    pub logs: DatabaseConfig,
    /// Admin/enforcement database
    pub admin: DatabaseConfig,
    /// Process database
    pub process: DatabaseConfig,
    /// Network database
    pub network: DatabaseConfig,
    /// Factor database
    pub factor: DatabaseConfig,
    /// ID database
    pub id: DatabaseConfig,
    /// Balance database
    pub balance: DatabaseConfig,
    /// Client database
    pub client: DatabaseConfig,
    /// Story database
    pub story: DatabaseConfig,
    /// Account database
    pub account: DatabaseConfig,
    /// Universe database
    pub universe: DatabaseConfig,
}

/// Individual database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)] // Don't serialize passwords
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub ssl_mode: String,
}

/// WebSocket server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub connection_timeout: u64,
    pub heartbeat_interval: u64,
    pub enable_compression: bool,
    pub max_message_size: usize,
    pub buffer_size: usize,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key (loaded from environment)
    #[serde(skip_serializing)]
    pub jwt_secret: String,
    /// JWT expiration in seconds
    pub jwt_expiration: u64,
    /// Enable refresh tokens
    pub enable_refresh_tokens: bool,
    /// Refresh token expiration in seconds
    pub refresh_token_expiration: u64,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Maximum sessions per user
    pub max_sessions_per_user: usize,
    /// Enable multi-factor authentication
    pub enable_mfa: bool,
    /// Enable OAuth providers
    pub enable_oauth: bool,
    /// Password requirements
    pub password_requirements: PasswordRequirements,
}

/// Password requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRequirements {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age_days: Option<u32>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Enable file logging
    pub enable_file: bool,
    /// Log file path
    pub file_path: Option<String>,
    /// Enable console logging
    pub enable_console: bool,
    /// Enable structured logging
    pub structured: bool,
    /// Log rotation settings
    pub rotation: LogRotationConfig,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size in MB
    pub max_size_mb: u64,
    /// Maximum number of files to keep
    pub max_files: u32,
    /// Enable compression of old files
    pub compress: bool,
}

/// Game-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Server name
    pub server_name: String,
    /// Maximum players online
    pub max_players: usize,
    /// Enable PvP
    pub enable_pvp: bool,
    /// Experience multiplier
    pub xp_multiplier: f64,
    /// Money multiplier
    pub money_multiplier: f64,
    /// Mission cooldown in seconds
    pub mission_cooldown: u64,
    /// Process timeout in seconds
    pub process_timeout: u64,
    /// Chat settings
    pub chat: ChatConfig,
    /// Newbie protection settings
    pub newbie_protection: NewbieProtectionConfig,
}

/// Chat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Enable global chat
    pub enable_global: bool,
    /// Maximum message length
    pub max_message_length: usize,
    /// Rate limit messages per minute
    pub rate_limit_per_minute: u32,
    /// Enable profanity filter
    pub enable_profanity_filter: bool,
}

/// Newbie protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewbieProtectionConfig {
    /// Enable newbie protection
    pub enable: bool,
    /// Protection duration in hours
    pub duration_hours: u32,
    /// Maximum level for protection
    pub max_level: u32,
}

/// Feature flags for gradual rollouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Map of feature names to enabled status
    pub flags: HashMap<String, bool>,
    /// Percentage-based rollouts
    pub rollouts: HashMap<String, f32>,
}

/// External services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    /// Redis configuration
    pub redis: Option<RedisConfig>,
    /// Email service configuration
    pub email: Option<EmailConfig>,
    /// Payment processor configuration
    pub payment: Option<PaymentConfig>,
    /// Analytics service configuration
    pub analytics: Option<AnalyticsConfig>,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

/// Email service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub provider: String,
    pub from_address: String,
    #[serde(skip_serializing)]
    pub api_key: String,
}

/// Payment processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub provider: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    pub webhook_secret: String,
}

/// Analytics service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub provider: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub enable_events: bool,
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable async processing
    pub enable_async: bool,
    /// Worker pool size
    pub worker_pool_size: usize,
    /// Buffer sizes
    pub buffer_sizes: BufferSizes,
    /// Cache settings
    pub cache: CacheConfig,
}

/// Buffer size configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSizes {
    pub websocket_buffer: usize,
    pub database_buffer: usize,
    pub log_buffer: usize,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enable: bool,
    /// Cache type (memory, redis)
    pub cache_type: String,
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Maximum cache size
    pub max_size: usize,
}

/// Configuration manager with hot-reloading support
#[derive(Debug)]
pub struct ConfigManager {
    config: Arc<RwLock<HeConfig>>,
    config_path: PathBuf,
    secret_manager: SecretManager,
    watchers: Vec<tokio::task::JoinHandle<()>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();
        let secret_manager = SecretManager::new().await?;
        
        let config = Self::load_config(&config_path, &secret_manager).await?;
        
        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            secret_manager,
            watchers: Vec::new(),
        };
        
        info!("Configuration manager initialized");
        Ok(manager)
    }

    /// Load configuration from file with environment overrides
    async fn load_config(config_path: &Path, secret_manager: &SecretManager) -> Result<HeConfig> {
        // Load base configuration from file
        let config_content = tokio::fs::read_to_string(config_path).await
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
        
        let mut config: HeConfig = match config_path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&config_content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&config_content)?,
            Some("toml") => toml::from_str(&config_content)?,
            _ => return Err(anyhow!("Unsupported config file format")),
        };

        // Apply environment overrides
        Self::apply_environment_overrides(&mut config).await?;
        
        // Load secrets
        Self::load_secrets(&mut config, secret_manager).await?;
        
        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Apply environment variable overrides
    async fn apply_environment_overrides(config: &mut HeConfig) -> Result<()> {
        // App configuration overrides
        if let Ok(env) = std::env::var("APP_ENVIRONMENT") {
            config.app.environment = match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "staging" | "stage" => Environment::Staging,
                "production" | "prod" => Environment::Production,
                _ => config.app.environment.clone(),
            };
        }

        if let Ok(port) = std::env::var("APP_PORT") {
            config.app.port = port.parse().unwrap_or(config.app.port);
        }

        if let Ok(debug) = std::env::var("APP_DEBUG") {
            config.app.debug = debug.parse().unwrap_or(config.app.debug);
        }

        // Database overrides (example for main database)
        if let Ok(db_host) = std::env::var("DB_HOST") {
            config.databases.main.host = db_host;
        }

        if let Ok(db_port) = std::env::var("DB_PORT") {
            config.databases.main.port = db_port.parse().unwrap_or(config.databases.main.port);
        }

        // WebSocket overrides
        if let Ok(ws_port) = std::env::var("WEBSOCKET_PORT") {
            config.websocket.port = ws_port.parse().unwrap_or(config.websocket.port);
        }

        debug!("Applied environment variable overrides");
        Ok(())
    }

    /// Load secrets from secure storage
    async fn load_secrets(config: &mut HeConfig, secret_manager: &SecretManager) -> Result<()> {
        // Load JWT secret
        if let Ok(jwt_secret) = secret_manager.get_secret("jwt_secret").await {
            config.auth.jwt_secret = jwt_secret;
        } else if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }

        // Load database passwords
        for (name, db_config) in [
            ("main", &mut config.databases.main),
            ("cache", &mut config.databases.cache),
            ("logs", &mut config.databases.logs),
            ("admin", &mut config.databases.admin),
            ("process", &mut config.databases.process),
            ("network", &mut config.databases.network),
            ("factor", &mut config.databases.factor),
            ("id", &mut config.databases.id),
            ("balance", &mut config.databases.balance),
            ("client", &mut config.databases.client),
            ("story", &mut config.databases.story),
            ("account", &mut config.databases.account),
            ("universe", &mut config.databases.universe),
        ] {
            let secret_key = format!("db_{}_password", name);
            if let Ok(password) = secret_manager.get_secret(&secret_key).await {
                db_config.password = password;
            } else if let Ok(password) = std::env::var(&secret_key.to_uppercase()) {
                db_config.password = password;
            }
        }

        debug!("Loaded secrets from secure storage");
        Ok(())
    }

    /// Get current configuration (read-only)
    pub async fn get_config(&self) -> HeConfig {
        self.config.read().await.clone()
    }

    /// Update configuration (for hot-reloading)
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = Self::load_config(&self.config_path, &self.secret_manager).await?;
        
        let mut config_guard = self.config.write().await;
        *config_guard = new_config;
        
        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// Start watching for configuration file changes
    pub async fn start_file_watcher(&mut self) -> Result<()> {
        use notify::{Watcher, RecursiveMode, watcher};
        use std::sync::mpsc;
        use std::time::Duration;

        let (tx, rx) = mpsc::channel();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;
        
        watcher.watch(&self.config_path, RecursiveMode::NonRecursive)?;

        let config_manager = self.config.clone();
        let config_path = self.config_path.clone();
        let secret_manager = self.secret_manager.clone();

        let handle = tokio::spawn(async move {
            let mut rx = tokio_stream::wrappers::ReceiverStream::new(
                tokio::sync::mpsc::unbounded_channel().1
            );

            while let Some(_event) = rx.next().await {
                match Self::load_config(&config_path, &secret_manager).await {
                    Ok(new_config) => {
                        let mut config_guard = config_manager.write().await;
                        *config_guard = new_config;
                        info!("Configuration auto-reloaded from file change");
                    }
                    Err(e) => {
                        error!("Failed to reload configuration: {}", e);
                    }
                }
            }
        });

        self.watchers.push(handle);
        info!("Configuration file watcher started");
        Ok(())
    }

    /// Check if a feature flag is enabled
    pub async fn is_feature_enabled(&self, feature_name: &str) -> bool {
        let config = self.config.read().await;
        config.features.flags.get(feature_name).copied().unwrap_or(false)
    }

    /// Check if a user is in a feature rollout
    pub async fn is_user_in_rollout(&self, feature_name: &str, user_id: &str) -> bool {
        let config = self.config.read().await;
        
        if let Some(rollout_percentage) = config.features.rollouts.get(feature_name) {
            let hash = Self::hash_user_id(user_id);
            let user_percentage = (hash % 100) as f32;
            user_percentage < *rollout_percentage
        } else {
            false
        }
    }

    /// Simple hash function for user ID (for consistent rollout assignment)
    fn hash_user_id(user_id: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        (hasher.finish() % 100) as u32
    }
}

/// Secret manager for secure credential storage
#[derive(Debug, Clone)]
pub struct SecretManager {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl SecretManager {
    /// Create a new secret manager
    pub async fn new() -> Result<Self> {
        Ok(Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get a secret by key
    pub async fn get_secret(&self, key: &str) -> Result<String> {
        let secrets = self.secrets.read().await;
        secrets.get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Secret not found: {}", key))
    }

    /// Set a secret
    pub async fn set_secret(&self, key: String, value: String) {
        let mut secrets = self.secrets.write().await;
        secrets.insert(key, value);
    }

    /// Load secrets from environment variables
    pub async fn load_from_env(&self) -> Result<()> {
        let mut secrets = self.secrets.write().await;
        
        for (key, value) in std::env::vars() {
            if key.ends_with("_SECRET") || key.ends_with("_PASSWORD") || key.ends_with("_KEY") {
                secrets.insert(key.to_lowercase(), value);
            }
        }
        
        info!("Loaded {} secrets from environment", secrets.len());
        Ok(())
    }
}

impl Default for HeConfig {
    fn default() -> Self {
        Self {
            app: AppConfig {
                name: "HackerExperience".to_string(),
                version: "2.0.0".to_string(),
                environment: Environment::Development,
                debug: true,
                base_url: "http://localhost:3000".to_string(),
                bind_address: "0.0.0.0".to_string(),
                port: 3000,
                workers: num_cpus::get(),
                request_timeout: 30,
                max_request_size: 10 * 1024 * 1024, // 10MB
            },
            databases: DatabasesConfig {
                main: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_main".to_string(),
                    max_connections: 20,
                    min_connections: 5,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                // ... initialize all other databases with similar defaults
                cache: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_cache".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                // ... other databases would be similar
                logs: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_log".to_string(),
                    max_connections: 15,
                    min_connections: 3,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                admin: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_henforcer".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                process: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_process".to_string(),
                    max_connections: 15,
                    min_connections: 3,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                network: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_network".to_string(),
                    max_connections: 15,
                    min_connections: 3,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                factor: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_factor".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                id: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_id".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                balance: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_balance".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                client: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_client".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                story: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_story".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                account: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_account".to_string(),
                    max_connections: 15,
                    min_connections: 3,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
                universe: DatabaseConfig {
                    host: "localhost".to_string(),
                    port: 3306,
                    username: "hackerexperience".to_string(),
                    password: "password".to_string(),
                    database: "helix_universe".to_string(),
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: 30,
                    idle_timeout: 600,
                    max_lifetime: 3600,
                    ssl_mode: "preferred".to_string(),
                },
            },
            websocket: WebSocketConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 4000,
                max_connections: 10000,
                connection_timeout: 60,
                heartbeat_interval: 30,
                enable_compression: true,
                max_message_size: 1024 * 1024, // 1MB
                buffer_size: 64 * 1024, // 64KB
            },
            auth: AuthConfig {
                jwt_secret: "change-this-in-production".to_string(),
                jwt_expiration: 3600,
                enable_refresh_tokens: true,
                refresh_token_expiration: 86400 * 7, // 7 days
                session_timeout: 3600,
                max_sessions_per_user: 5,
                enable_mfa: false,
                enable_oauth: false,
                password_requirements: PasswordRequirements {
                    min_length: 8,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_numbers: true,
                    require_symbols: false,
                    max_age_days: None,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                enable_file: true,
                file_path: Some("logs/hackerexperience.log".to_string()),
                enable_console: true,
                structured: false,
                rotation: LogRotationConfig {
                    max_size_mb: 100,
                    max_files: 10,
                    compress: true,
                },
            },
            game: GameConfig {
                server_name: "HackerExperience Server".to_string(),
                max_players: 1000,
                enable_pvp: true,
                xp_multiplier: 1.0,
                money_multiplier: 1.0,
                mission_cooldown: 300, // 5 minutes
                process_timeout: 3600, // 1 hour
                chat: ChatConfig {
                    enable_global: true,
                    max_message_length: 500,
                    rate_limit_per_minute: 30,
                    enable_profanity_filter: true,
                },
                newbie_protection: NewbieProtectionConfig {
                    enable: true,
                    duration_hours: 72,
                    max_level: 10,
                },
            },
            features: FeatureFlags {
                flags: HashMap::new(),
                rollouts: HashMap::new(),
            },
            services: ServicesConfig {
                redis: None,
                email: None,
                payment: None,
                analytics: None,
            },
            performance: PerformanceConfig {
                enable_async: true,
                worker_pool_size: num_cpus::get() * 2,
                buffer_sizes: BufferSizes {
                    websocket_buffer: 64 * 1024,
                    database_buffer: 128 * 1024,
                    log_buffer: 32 * 1024,
                },
                cache: CacheConfig {
                    enable: true,
                    cache_type: "memory".to_string(),
                    default_ttl: 3600,
                    max_size: 1000,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = HeConfig::default();
        assert_eq!(config.app.name, "HackerExperience");
        assert_eq!(config.app.environment, Environment::Development);
        assert!(config.app.debug);
    }

    #[tokio::test]
    async fn test_secret_manager() {
        let secret_manager = SecretManager::new().await.unwrap();
        
        secret_manager.set_secret("test_key".to_string(), "test_value".to_string()).await;
        
        let value = secret_manager.get_secret("test_key").await.unwrap();
        assert_eq!(value, "test_value");
        
        let missing = secret_manager.get_secret("missing_key").await;
        assert!(missing.is_err());
    }

    #[tokio::test]
    async fn test_config_loading() {
        let config = HeConfig::default();
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        
        let config_manager = ConfigManager::new(temp_file.path()).await.unwrap();
        let loaded_config = config_manager.get_config().await;
        
        assert_eq!(loaded_config.app.name, "HackerExperience");
    }
}