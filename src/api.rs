use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::library::{LibraryCache, LibraryMetadata};
use crate::db::Book;
use crate::config;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FilterQuery {
    pub formats: Option<Vec<String>>,
    pub search: Option<String>,
}

pub async fn get_libraries(
    cache: web::Data<Mutex<LibraryCache>>,
) -> Result<HttpResponse> {
    let cache = cache.lock().unwrap();
    let libraries = cache.get_libraries();
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(libraries),
        error: None,
    }))
}

pub async fn get_library(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let library_id = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(lib) = cache.get_library(&library_id) {
        Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(lib.clone()),
            error: None,
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<LibraryMetadata> {
            success: false,
            data: None,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_books(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<String>,
    query: web::Query<FilterQuery>,
) -> Result<HttpResponse> {
    let library_id = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(db) = cache.get_database(&library_id) {
        match db.get_all_books() {
            Ok(mut books) => {
                // Apply search filter if provided
                if let Some(search_term) = &query.search {
                    let search_lower = search_term.to_lowercase();
                    books.retain(|book| {
                        book.title.to_lowercase().contains(&search_lower)
                            || book.authors.iter().any(|a| a.to_lowercase().contains(&search_lower))
                    });
                }

                // Apply format filter if provided
                if let Some(requested_formats) = &query.formats {
                    let requested_formats_upper: Vec<String> = 
                        requested_formats.iter().map(|f| f.to_uppercase()).collect();
                    
                    books.retain(|book| {
                        match db.get_book_formats(book.id) {
                            Ok(available_formats) => {
                                available_formats.iter().any(|fmt| {
                                    requested_formats_upper.contains(&fmt.to_uppercase())
                                })
                            }
                            Err(_) => false,
                        }
                    });
                }

                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(books),
                    error: None,
                }))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<Vec<Book>> {
                    success: false,
                    data: None,
                    error: Some(format!("Database error: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<Vec<Book>> {
            success: false,
            data: None,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_book(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<(String, i32)>,
) -> Result<HttpResponse> {
    let (library_id, book_id) = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(db) = cache.get_database(&library_id) {
        match db.get_book(book_id) {
            Ok(Some(book)) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(book),
                    error: None,
                }))
            }
            Ok(None) => {
                Ok(HttpResponse::NotFound().json(ApiResponse::<Book> {
                    success: false,
                    data: None,
                    error: Some("Book not found".to_string()),
                }))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<Book> {
                    success: false,
                    data: None,
                    error: Some(format!("Database error: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<Book> {
            success: false,
            data: None,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_authors(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let library_id = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(db) = cache.get_database(&library_id) {
        match db.get_all_authors() {
            Ok(authors) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(authors),
                    error: None,
                }))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse {
                    success: false,
                    data: None as Option<Vec<String>>,
                    error: Some(format!("Database error: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            data: None as Option<Vec<String>>,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_tags(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let library_id = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(db) = cache.get_database(&library_id) {
        match db.get_all_tags() {
            Ok(tags) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(tags),
                    error: None,
                }))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse {
                    success: false,
                    data: None as Option<Vec<String>>,
                    error: Some(format!("Database error: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            data: None as Option<Vec<String>>,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_series(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let library_id = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(db) = cache.get_database(&library_id) {
        match db.get_all_series() {
            Ok(series) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(series),
                    error: None,
                }))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse {
                    success: false,
                    data: None as Option<Vec<String>>,
                    error: Some(format!("Database error: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            data: None as Option<Vec<String>>,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_book_cover(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<(String, i32)>,
) -> Result<HttpResponse> {
    let (library_id, book_id) = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(lib) = cache.get_library(&library_id) {
        // Find book directory by searching for pattern "*({book_id})"
        // Calibre stores books as: {library}/{author}/{book_title ({book_id})}/cover.jpg
        let pattern = format!("({})", book_id);
        
        if let Ok(entries) = std::fs::read_dir(&lib.path) {
            for author_dir in entries.flatten() {
                let author_path = author_dir.path();
                if author_path.is_dir() {
                    if let Ok(book_entries) = std::fs::read_dir(&author_path) {
                        for book_dir in book_entries.flatten() {
                            let book_path = book_dir.path();
                            if book_path.is_dir() {
                                let dir_name = book_path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");
                                
                                if dir_name.ends_with(&pattern) {
                                    let cover_path = book_path.join("cover.jpg");
                                    if cover_path.exists() {
                                        if let Ok(data) = std::fs::read(&cover_path) {
                                            return Ok(HttpResponse::Ok()
                                                .content_type("image/jpeg")
                                                .body(data));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // If not found, return 404
        Ok(HttpResponse::NotFound().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub async fn get_book_formats(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<(String, i32)>,
) -> Result<HttpResponse> {
    let (library_id, book_id) = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(db) = cache.get_database(&library_id) {
        match db.get_book_formats(book_id) {
            Ok(formats) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(formats),
                    error: None,
                }))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<Vec<String>> {
                    success: false,
                    data: None,
                    error: Some(format!("Database error: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<Vec<String>> {
            success: false,
            data: None,
            error: Some("Library not found".to_string()),
        }))
    }
}

pub async fn get_book_file(
    cache: web::Data<Mutex<LibraryCache>>,
    path: web::Path<(String, i32, String)>,
) -> Result<HttpResponse> {
    let (library_id, book_id, format) = path.into_inner();
    let cache = cache.lock().unwrap();
    
    if let Some(lib) = cache.get_library(&library_id) {
        // Find book directory by searching for pattern "*({book_id})"
        // Calibre stores books as: {library}/{author}/{book_title ({book_id})}/
        let pattern = format!("({})", book_id);
        let format_upper = format.to_uppercase();
        
        if let Ok(entries) = std::fs::read_dir(&lib.path) {
            for author_dir in entries.flatten() {
                let author_path = author_dir.path();
                if author_path.is_dir() {
                    if let Ok(book_entries) = std::fs::read_dir(&author_path) {
                        for book_dir in book_entries.flatten() {
                            let book_path = book_dir.path();
                            if book_path.is_dir() {
                                let dir_name = book_path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");
                                
                                if dir_name.ends_with(&pattern) {
                                    // Look for the file with the matching format
                                    if let Ok(file_entries) = std::fs::read_dir(&book_path) {
                                        for file_entry in file_entries.flatten() {
                                            let file_path = file_entry.path();
                                            if file_path.is_file() {
                                                if let Some(ext) = file_path.extension() {
                                                    if ext.to_string_lossy().to_uppercase() == format_upper {
                                                        if let Ok(data) = std::fs::read(&file_path) {
                                                            let content_type = match format_upper.as_str() {
                                                                "EPUB" => "application/epub+zip",
                                                                "PDF" => "application/pdf",
                                                                "MOBI" => "application/x-mobipocket-ebook",
                                                                "AZW" => "application/vnd.amazon.ebook",
                                                                "AZW3" => "application/vnd.amazon.ebook",
                                                                "HTML" => "text/html",
                                                                "TXT" => "text/plain; charset=utf-8",
                                                                _ => "application/octet-stream",
                                                            };
                                                            
                                                            let filename = file_path.file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("book");
                                                            
                                                            return Ok(HttpResponse::Ok()
                                                                .content_type(content_type)
                                                                .insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", filename)))
                                                                .body(data));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // If not found, return 404
        Ok(HttpResponse::NotFound().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub async fn refresh_libraries(
    cache: web::Data<Mutex<LibraryCache>>,
) -> Result<HttpResponse> {
    let mut cache = cache.lock().unwrap();
    let libraries_path = std::path::Path::new(config::LIBRARY_PATH);
    
    // Clear and reload the cache
    cache.clear();
    
    if libraries_path.exists() {
        match cache.load_libraries(libraries_path) {
            Ok(_) => {
                let libraries = cache.get_libraries();
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(libraries),
                    error: None,
                }))
            },
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(ApiResponse {
                    success: false,
                    data: None::<Vec<LibraryMetadata>>,
                    error: Some(format!("Failed to refresh libraries: {}", e)),
                }))
            }
        }
    } else {
        Ok(HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            data: None::<Vec<LibraryMetadata>>,
            error: Some("Libraries path not found".to_string()),
        }))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/libraries", web::get().to(get_libraries))
            .route("/libraries/refresh", web::post().to(refresh_libraries))
            .route("/libraries/{id}", web::get().to(get_library))
            .route("/libraries/{id}/books", web::get().to(get_books))
            .route("/libraries/{id}/authors", web::get().to(get_authors))
            .route("/libraries/{id}/tags", web::get().to(get_tags))
            .route("/libraries/{id}/series", web::get().to(get_series))
            .route("/libraries/{id}/books/{book_id}", web::get().to(get_book))
            .route("/libraries/{id}/books/{book_id}/cover", web::get().to(get_book_cover))
            .route("/libraries/{id}/books/{book_id}/formats", web::get().to(get_book_formats))
            .route("/libraries/{id}/books/{book_id}/formats/{format}", web::get().to(get_book_file))
    );
}
