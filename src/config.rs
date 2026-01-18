/// Configuration module for Biblio
/// 
/// This module loads configuration from a YAML file at runtime.
/// The configuration file should be located at `config.yaml` in the working directory.
/// See `config.yaml.example` for setup instructions.

use serde::{Deserialize, Serialize};
use std::fs;
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
    /// Load configuration from the YAML file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml_ng::from_str(&contents)?;
        Ok(config)
    }
}

/// Global configuration instance (thread-safe)
static CONFIG: std::sync::OnceLock<Arc<Mutex<Config>>> = std::sync::OnceLock::new();

/// Initialize the global configuration from the YAML file
pub fn init(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(config_path)?;
    info!("Configuration loaded from {}", config_path);
    
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
