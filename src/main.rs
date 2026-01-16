mod db;
mod library;
mod api;
mod config;

use actix_web::{web, App, HttpServer, middleware};
use actix_files::Files;
use library::LibraryCache;
use std::sync::Mutex;
use std::path::Path;
use tracing::{warn, info};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

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
            .wrap(middleware::Logger::default())
            .configure(api::configure)
            .service(Files::new("/", "./public").index_file("index.html"))
    })
    .bind(service_ip_and_port)?
    .run()
    .await
}
