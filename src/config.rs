/// Configuration module for Biblio
/// 
/// This module loads configuration from a YAML file at runtime.
/// 
/// Location:
/// - If APP_IN_DOCKER environment variable is set to "true", config.yaml is expected
///   at `/config/config.yaml` (a mounted volume in Docker)
/// - Otherwise, config.yaml is expected in the current working directory
/// 
/// Path Resolution:
/// - Relative paths in config.yaml are resolved relative to the base directory
///   (either `/config` for Docker or current directory for standard Linux)
/// - Absolute paths are used as-is
/// 
/// See `config.yaml.example` for setup instructions.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Runtime configuration loaded from config.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the Calibre libraries directory
    pub library_path: String,
    
    /// IP and port for the Biblio server to bind to
    pub service_ip_and_port: String,
    
    /// Path to the users.ids file containing user credentials
    pub users_file_path: String,
    
    /// Enable HTTPS/TLS
    pub use_https: bool,
    
    /// Path to the SSL/TLS certificate file (PEM format)
    pub certificate_path: String,
    
    /// Path to the SSL/TLS private key file (PEM format)
    pub private_key_path: String,
}

impl Config {
    /// Determine the base directory for path resolution
    fn get_base_dir() -> PathBuf {
        if std::env::var("APP_IN_DOCKER").unwrap_or_default() == "true" {
            PathBuf::from("/config")
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    }

    /// Resolve a path: if relative, make it relative to base_dir; if absolute, use as-is
    fn resolve_path(base_dir: &PathBuf, path: &str) -> String {
        let p = PathBuf::from(path);
        if p.is_absolute() {
            path.to_string()
        } else {
            base_dir.join(p).to_string_lossy().to_string()
        }
    }

    /// Load configuration from the YAML file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml_ng::from_str(&contents)?;
        
        // Resolve relative paths
        let base_dir = Self::get_base_dir();
        config.library_path = Self::resolve_path(&base_dir, &config.library_path);
        config.users_file_path = Self::resolve_path(&base_dir, &config.users_file_path);
        config.certificate_path = Self::resolve_path(&base_dir, &config.certificate_path);
        config.private_key_path = Self::resolve_path(&base_dir, &config.private_key_path);
        
        Ok(config)
    }
}

/// Global configuration instance (thread-safe)
static CONFIG: std::sync::OnceLock<Arc<Mutex<Config>>> = std::sync::OnceLock::new();

/// Initialize the global configuration from the YAML file
pub fn init(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(config_path)?;
    info!("Configuration loaded from {}", config_path);
    
    // Determine and log which mode we're running in
    if std::env::var("APP_IN_DOCKER").unwrap_or_default() == "true" {
        info!("Running in Docker mode (APP_IN_DOCKER=true)");
    } else {
        info!("Running in standard mode (APP_IN_DOCKER not set or false)");
    }
    
    // Validate configuration paths exist or warn if they don't
    if !std::path::Path::new(&config.library_path).exists() {
        warn!("Library path does not exist: {}", config.library_path);
    }
    
    if config.use_https {
        if !std::path::Path::new(&config.certificate_path).exists() {
            warn!("Certificate file not found: {}", config.certificate_path);
        }
        if !std::path::Path::new(&config.private_key_path).exists() {
            warn!("Private key file not found: {}", config.private_key_path);
        }
    }
    
    CONFIG.get_or_init(|| Arc::new(Mutex::new(config)));
    Ok(())
}

/// Get the current configuration
pub fn get() -> Arc<Mutex<Config>> {
    CONFIG
        .get()
        .cloned()
        .unwrap_or_else(|| {
            error!("Configuration not initialized. Call config::init() during startup.");
            panic!("Configuration not initialized");
        })
}

/// Convenience function to get a config value (immutable access)
pub fn with<F, R>(f: F) -> R
where
    F: FnOnce(&Config) -> R,
{
    let config = get();
    let cfg = config.lock().unwrap();
    f(&cfg)
}

/// Convenience accessors for direct configuration values
pub fn library_path() -> String {
    with(|cfg| cfg.library_path.clone())
}

pub fn service_ip_and_port() -> String {
    with(|cfg| cfg.service_ip_and_port.clone())
}

pub fn users_file_path() -> String {
    with(|cfg| cfg.users_file_path.clone())
}

pub fn use_https() -> bool {
    with(|cfg| cfg.use_https)
}

pub fn certificate_path() -> String {
    with(|cfg| cfg.certificate_path.clone())
}

pub fn private_key_path() -> String {
    with(|cfg| cfg.private_key_path.clone())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_config_load() {
        // This test would require a test config.yaml file
        // For now, we just verify the functions exist and compile
    }
}
