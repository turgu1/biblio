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

    info!("Starting Biblio server on http://{}", service_ip_and_port);

    HttpServer::new(move || {
        App::new()
            .app_data(cache.clone())
            .app_data(users.clone())
            .app_data(session_store.clone())
            .app_data(audit_logger.clone())
            .wrap(middleware::Logger::default())
            .configure(api::configure)
            .service(Files::new("/", "./public").index_file("index.html"))
    })
    .bind(service_ip_and_port)?
    .run()
    .await
}
