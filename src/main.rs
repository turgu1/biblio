mod db;
mod library;
mod api;
mod config;
mod auth;
mod session;
mod audit;
mod rbac;

use actix_web::{web, App, HttpServer, middleware};
use actix_files::Files;
use library::LibraryCache;
use std::sync::Mutex;
use std::path::Path;
use tracing::{warn, info, error};
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls_pemfile::{read_one, Item};
use std::fs::File;
use std::io::BufReader;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load users for authentication
    let users = match auth::load_users(config::USERS_FILE_PATH) {
        Ok(users) => {
            info!("Loaded {} user(s) for authentication", users.len());
            web::Data::new(users)
        }
        Err(e) => {
            error!("Failed to load users: {}", e);
            error!("Authentication will be disabled. Configure USERS_FILE_PATH in src/config.rs");
            web::Data::new(vec![])
        }
    };

    // Initialize session store (30 minute timeout)
    let session_store = web::Data::new(session::SessionStore::new(30));

    // Initialize audit logger (keep last 1000 events)
    let audit_logger = web::Data::new(audit::AuditLogger::new(1000));

    // Set up library cache
    let mut cache = LibraryCache::new();
    
    // Load libraries from the configured path
    let libraries_path = Path::new(config::LIBRARY_PATH);
    if libraries_path.exists() {
        cache.load_libraries(libraries_path)?;
    } else {
        warn!("Libraries directory not found at {:?}", libraries_path);
        warn!("Please configure LIBRARY_PATH in src/config.rs");
    }

    let cache = web::Data::new(Mutex::new(cache));

    // Load libraries from the configured path
    let service_ip_and_port = config::SERVICE_IP_AND_PORT;

    // Determine protocol and log startup info
    let protocol = if config::USE_HTTPS { "https" } else { "http" };
    info!("Starting Biblio server on {}://{}", protocol, service_ip_and_port);

    let server_builder = HttpServer::new(move || {
        App::new()
            .app_data(cache.clone())
            .app_data(users.clone())
            .app_data(session_store.clone())
            .app_data(audit_logger.clone())
            .wrap(middleware::Logger::default())
            .configure(api::configure)
            .service(Files::new("/", "./public").index_file("index.html"))
    });

    if config::USE_HTTPS {
        info!("Setting up HTTPS with TLS");
        match load_tls_config() {
            Ok(tls_config) => {
                info!("TLS configuration loaded successfully");
                server_builder
                    .bind_rustls(service_ip_and_port, tls_config)?
                    .run()
                    .await
            }
            Err(e) => {
                error!("Failed to load TLS configuration: {}", e);
                error!("Falling back to HTTP");
                server_builder.bind(service_ip_and_port)?.run().await
            }
        }
    } else {
        server_builder.bind(service_ip_and_port)?.run().await
    }
}

// Helper function to load TLS configuration from certificate and key files
fn load_tls_config() -> std::io::Result<ServerConfig> {
    // Load certificates
    let cert_file = File::open(config::CERTIFICATE_PATH)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to open certificate file at {}: {}", config::CERTIFICATE_PATH, e)
        ))?;
    let mut cert_reader = BufReader::new(cert_file);
    
    let mut certs = Vec::new();
    loop {
        match read_one(&mut cert_reader) {
            Ok(Some(Item::X509Certificate(cert))) => {
                certs.push(Certificate(cert));
            }
            Ok(None) => break,
            Ok(_) => continue,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse certificate file: {}", e)
                ));
            }
        }
    }

    if certs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No certificates found in certificate file"
        ));
    }

    // Load private key
    let key_file = File::open(config::PRIVATE_KEY_PATH)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to open private key file at {}: {}", config::PRIVATE_KEY_PATH, e)
        ))?;
    let mut key_reader = BufReader::new(key_file);
    
    let mut private_key: Option<PrivateKey> = None;
    loop {
        match read_one(&mut key_reader) {
            Ok(Some(Item::RSAKey(key))) => {
                private_key = Some(PrivateKey(key));
                break;
            }
            Ok(Some(Item::PKCS8Key(key))) => {
                private_key = Some(PrivateKey(key));
                break;
            }
            Ok(None) => break,
            Ok(_) => continue,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse private key file: {}", e)
                ));
            }
        }
    }

    let private_key = private_key
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No private key found in key file"
        ))?;

    // Build and return the TLS configuration
    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to build TLS configuration: {}", e)
        ))
}