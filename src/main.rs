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
use rustls::{ServerConfig, pki_types::CertificateDer};
use rustls_pemfile::certs;
use std::fs::File;
use std::io::BufReader;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize configuration from YAML file
    if let Err(e) = config::init("config.yaml") {
        error!("Failed to initialize configuration: {}", e);
        error!("Ensure config.yaml exists in the working directory. Copy config.yaml.example to get started.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Configuration initialization failed: {}", e)
        ));
    }

    // Load users for authentication
    let users_path = config::users_file_path();
    let users = match auth::load_users(&users_path) {
        Ok(users) => {
            info!("Loaded {} user(s) for authentication", users.len());
            web::Data::new(users)
        }
        Err(e) => {
            error!("Failed to load users: {}", e);
            error!("Authentication will be disabled. Configure users_file_path in config.yaml");
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
    let library_path = config::library_path();
    let libraries_path = Path::new(&library_path);
    if libraries_path.exists() {
        cache.load_libraries(libraries_path)?;
    } else {
        warn!("Libraries directory not found at {:?}", libraries_path);
        warn!("Please configure library_path in config.yaml");
    }

    let cache = web::Data::new(Mutex::new(cache));

    // Get service binding address
    let service_ip_and_port = config::service_ip_and_port();

    // Determine protocol and log startup info
    let protocol = if config::use_https() { "https" } else { "http" };
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

    if config::use_https() {
        info!("Setting up HTTPS with TLS");
        match load_tls_config() {
            Ok(tls_config) => {
                info!("TLS configuration loaded successfully");
                server_builder
                    .bind_rustls_0_23(&service_ip_and_port, tls_config)?
                    .run()
                    .await
            }
            Err(e) => {
                error!("Failed to load TLS configuration: {}", e);
                error!("Falling back to HTTP");
                server_builder.bind(&service_ip_and_port)?.run().await
            }
        }
    } else {
        server_builder.bind(&service_ip_and_port)?.run().await
    }
}

// Helper function to load TLS configuration from certificate and key files
fn load_tls_config() -> std::io::Result<ServerConfig> {
    let cert_path = config::certificate_path();
    let key_path = config::private_key_path();
    
    // Load certificates
    let cert_file = File::open(&cert_path)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to open certificate file at {}: {}", cert_path, e)
        ))?;
    let mut cert_reader = BufReader::new(cert_file);
    
    let certs_vec: Vec<CertificateDer> = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse certificate file: {}", e)
        ))?;
    
    if certs_vec.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No certificates found in certificate file"
        ));
    }

    // Load private key
    let key_file = File::open(&key_path)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to open private key file at {}: {}", key_path, e)
        ))?;
    let mut key_reader = BufReader::new(key_file);
    
    // Try to read private key - rustls-pemfile 2.x provides specific readers
    let key_bytes = std::io::read_to_string(&mut key_reader)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to read private key file: {}", e)
        ))?;
    
    let private_key = rustls_pemfile::private_key(&mut std::io::Cursor::new(key_bytes))
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse private key file: {}", e)
        ))?
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No private key found in key file"
        ))?;

    // Build and return the TLS configuration
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs_vec, private_key)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to build TLS configuration: {}", e)
        ))
}