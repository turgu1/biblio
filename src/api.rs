use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::library::{LibraryCache, LibraryMetadata};
use crate::db::Book;
use crate::config;
use crate::auth;
#[allow(unused_imports)]
use crate::session;
#[allow(unused_imports)]
use crate::audit;
#[allow(unused_imports)]
use crate::rbac;

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

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub username: String,
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub username: String,
    pub reset_token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub username: Option<String>,
    pub limit: Option<usize>,
}

// User management request/response structures
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>, // admin, librarian, user, reader (default: reader)
    pub email: Option<String>,
    pub admin_username: Option<String>, // Username of the admin making this request
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<String>,
    pub email: Option<String>,
    pub admin_username: Option<String>, // Username of the admin making this request
}

#[derive(Debug, Deserialize)]
pub struct AdminChangePasswordRequest {
    pub username: String,
    pub new_password: String,
    pub admin_username: Option<String>, // Username of the admin making this request
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserRequest {
    pub admin_username: Option<String>, // Username of the admin making this request
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub username: String,
    pub role: String,
    pub email: Option<String>,
    pub created_at: Option<String>,
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

pub async fn login(
    req: web::Json<LoginRequest>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    // Load users from file to get the latest data (including newly created users)
    let file_users = match auth::load_users(config::USERS_FILE_PATH) {
        Ok(users) => users,
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::LoginFailure,
                &req.username,
                "127.0.0.1",
                &format!("Error loading users: {}", e),
                false,
            );
            
            return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Authentication system error: {}", e)),
            }));
        }
    };

    // Validate username and password
    match auth::authenticate_user(&req.username, &req.password, &file_users) {
        Ok(true) => {
            // Find the user to get their role
            let user_role = file_users.iter()
                .find(|u| u.username == req.username)
                .map(|u| u.role.clone())
                .unwrap_or_else(|| "reader".to_string());
            
            audit_logger.log_event(
                audit::AuditEventType::LoginSuccess,
                &req.username,
                "127.0.0.1",
                "User logged in successfully",
                true,
            );
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({"username": req.username, "role": user_role})),
                error: None,
            }))
        }
        Ok(false) => {
            audit_logger.log_event(
                audit::AuditEventType::LoginFailure,
                &req.username,
                "127.0.0.1",
                "Invalid credentials",
                false,
            );
            
            Ok(HttpResponse::Unauthorized().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some("Invalid credentials".to_string()),
            }))
        }
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::LoginFailure,
                &req.username,
                "127.0.0.1",
                &format!("Authentication error: {}", e),
                false,
            );
            
            Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Authentication error: {}", e)),
            }))
        }
    }
}

pub async fn logout(
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    audit_logger.log_event(
        audit::AuditEventType::LogoutSuccess,
        "unknown",
        "127.0.0.1",
        "User logged out",
        true,
    );
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({"message": "logged out"})),
        error: None,
    }))
}

pub async fn get_current_user(
    _users: web::Data<Vec<auth::User>>,
) -> Result<HttpResponse> {
    // Return a generic user response - the frontend maintains auth state via localStorage
    // In a production system with session/JWT, this would return the actual logged-in user
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "username": "user",
            "role": "reader",
            "email": null
        })),
        error: None,
    }))
}

// Password Management Endpoints

pub async fn change_password(
    req: web::Json<ChangePasswordRequest>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    // Validate new password strength
    if let Err(e) = auth::validate_password_strength(&req.new_password) {
        audit_logger.log_event(
            audit::AuditEventType::PasswordChange,
            &req.username,
            "127.0.0.1",
            &format!("Failed: {}", e),
            false,
        );
        
        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some(e),
        }));
    }

    // Load users from file
    let mut file_users = match auth::load_users(config::USERS_FILE_PATH) {
        Ok(users) => users,
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::PasswordChange,
                &req.username,
                "127.0.0.1",
                &format!("Failed: {}", e),
                false,
            );

            return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error loading users: {}", e)),
            }));
        }
    };

    // Authenticate user with current password
    match auth::authenticate_user(&req.username, &req.current_password, &file_users) {
        Ok(true) => {
            // Hash the new password
            match auth::hash_password(&req.new_password) {
                Ok(new_hash) => {
                    // Find and update the user
                    if let Some(user) = file_users.iter_mut().find(|u| u.username == req.username) {
                        user.password_hash = new_hash;

                        // Save updated users to file
                        if let Err(e) = auth::save_users(&file_users, config::USERS_FILE_PATH) {
                            audit_logger.log_event(
                                audit::AuditEventType::PasswordChange,
                                &req.username,
                                "127.0.0.1",
                                &format!("Failed to save: {}", e),
                                false,
                            );

                            return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                                success: false,
                                data: None::<serde_json::Value>,
                                error: Some(format!("Error saving password: {}", e)),
                            }));
                        }

                        audit_logger.log_event(
                            audit::AuditEventType::PasswordChange,
                            &req.username,
                            "127.0.0.1",
                            "Password changed successfully",
                            true,
                        );

                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            data: Some(serde_json::json!({"message": "Password changed successfully"})),
                            error: None,
                        }))
                    } else {
                        audit_logger.log_event(
                            audit::AuditEventType::PasswordChange,
                            &req.username,
                            "127.0.0.1",
                            "Failed: User not found",
                            false,
                        );

                        Ok(HttpResponse::NotFound().json(ApiResponse {
                            success: false,
                            data: None::<serde_json::Value>,
                            error: Some("User not found".to_string()),
                        }))
                    }
                }
                Err(e) => {
                    audit_logger.log_event(
                        audit::AuditEventType::PasswordChange,
                        &req.username,
                        "127.0.0.1",
                        &format!("Failed to hash password: {}", e),
                        false,
                    );

                    Ok(HttpResponse::InternalServerError().json(ApiResponse {
                        success: false,
                        data: None::<serde_json::Value>,
                        error: Some(format!("Error processing password: {}", e)),
                    }))
                }
            }
        }
        Ok(false) => {
            audit_logger.log_event(
                audit::AuditEventType::PasswordChange,
                &req.username,
                "127.0.0.1",
                "Failed: Invalid current password",
                false,
            );

            Ok(HttpResponse::Unauthorized().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some("Invalid current password".to_string()),
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error: {}", e)),
            }))
        }
    }
}


// User Management Endpoints (Admin Only)

/// Helper function to verify if a user has admin role
fn verify_admin_user(admin_username: Option<&str>) -> Result<(), String> {
    let username = admin_username.ok_or("Admin username required")?;
    
    let file_users = auth::load_users(config::USERS_FILE_PATH)
        .map_err(|e| format!("Failed to load users: {}", e))?;
    
    let user = file_users.iter()
        .find(|u| u.username == username)
        .ok_or("Admin user not found")?;
    
    if user.role != "admin" {
        return Err("User does not have admin role".to_string());
    }
    
    Ok(())
}

pub async fn list_users(
    query: web::Query<std::collections::HashMap<String, String>>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    // Verify admin role (get from query parameter)
    let admin_username = query.get("admin_username").map(|s| s.as_str());
    
    if let Err(e) = verify_admin_user(admin_username) {
        audit_logger.log_event(
            audit::AuditEventType::UnauthorizedAccess,
            admin_username.unwrap_or("unknown"),
            "127.0.0.1",
            &format!("Unauthorized attempt to list users: {}", e),
            false,
        );

        return Ok(HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            data: None::<Vec<UserResponse>>,
            error: Some("Unauthorized: Admin access required".to_string()),
        }));
    }

    // Read users from file to get the latest data (including newly created users)
    match auth::load_users(config::USERS_FILE_PATH) {
        Ok(file_users) => {
            let user_responses: Vec<UserResponse> = file_users
                .iter()
                .map(|u| UserResponse {
                    username: u.username.clone(),
                    role: u.role.clone(),
                    email: u.email.clone(),
                    created_at: u.created_at.clone(),
                })
                .collect();

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(user_responses),
                error: None,
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<Vec<UserResponse>>,
                error: Some(format!("Error loading users: {}", e)),
            }))
        }
    }
}

pub async fn create_user(
    req: web::Json<CreateUserRequest>,
    users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    // Verify admin role
    if let Err(e) = verify_admin_user(req.admin_username.as_deref()) {
        audit_logger.log_event(
            audit::AuditEventType::UnauthorizedAccess,
            req.admin_username.as_deref().unwrap_or("unknown"),
            "127.0.0.1",
            &format!("Unauthorized attempt to create user: {}", e),
            false,
        );

        return Ok(HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("Unauthorized: Admin access required".to_string()),
        }));
    }

    // Validate password strength
    if let Err(e) = auth::validate_password_strength(&req.password) {
        audit_logger.log_event(
            audit::AuditEventType::UserCreated,
            req.admin_username.as_deref().unwrap_or("admin"),
            "127.0.0.1",
            &format!("Failed to create user {}: {}", req.username, e),
            false,
        );

        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some(e),
        }));
    }

    // Check if user already exists
    if users.iter().any(|u| u.username == req.username) {
        audit_logger.log_event(
            audit::AuditEventType::UserCreated,
            "admin",
            "127.0.0.1",
            &format!("Failed to create user {}: user already exists", req.username),
            false,
        );

        return Ok(HttpResponse::Conflict().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("User already exists".to_string()),
        }));
    }

    // Hash the password
    match auth::hash_password(&req.password) {
        Ok(password_hash) => {
            let role = req.role.clone().unwrap_or_else(|| "reader".to_string());
            let created_at = chrono::Utc::now().to_rfc3339();
            
            // Create the new user
            let new_user = auth::User {
                username: req.username.clone(),
                password_hash,
                role: role.clone(),
                email: req.email.clone(),
                created_at: Some(created_at.clone()),
            };

            // Load all users from file (not from cache), add the new one, and save back to file
            let mut all_users = match auth::load_users(config::USERS_FILE_PATH) {
                Ok(users) => users,
                Err(e) => {
                    audit_logger.log_event(
                        audit::AuditEventType::UserCreated,
                        req.admin_username.as_deref().unwrap_or("admin"),
                        "127.0.0.1",
                        &format!("Failed to load users from file: {}", e),
                        false,
                    );
                    
                    return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                        success: false,
                        data: None::<serde_json::Value>,
                        error: Some(format!("Error loading users: {}", e)),
                    }));
                }
            };

            all_users.push(new_user);

            // Save to file
            if let Err(e) = auth::save_users(&all_users, config::USERS_FILE_PATH) {
                audit_logger.log_event(
                    audit::AuditEventType::UserCreated,
                    "admin",
                    "127.0.0.1",
                    &format!("Failed to save user {} to file: {}", req.username, e),
                    false,
                );

                return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                    success: false,
                    data: None::<serde_json::Value>,
                    error: Some(format!("Error saving user to file: {}", e)),
                }));
            }

            audit_logger.log_event(
                audit::AuditEventType::UserCreated,
                "admin",
                "127.0.0.1",
                &format!("Created user {} with role {}", req.username, role),
                true,
            );

            let user_response = UserResponse {
                username: req.username.clone(),
                role,
                email: req.email.clone(),
                created_at: Some(created_at),
            };

            Ok(HttpResponse::Created().json(ApiResponse {
                success: true,
                data: Some(user_response),
                error: None,
            }))
        }
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::UserCreated,
                "admin",
                "127.0.0.1",
                &format!("Failed to create user {}: {}", req.username, e),
                false,
            );

            Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error creating user: {}", e)),
            }))
        }
    }
}

pub async fn update_user(
    path: web::Path<String>,
    req: web::Json<UpdateUserRequest>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    // Verify admin role
    if let Err(e) = verify_admin_user(req.admin_username.as_deref()) {
        audit_logger.log_event(
            audit::AuditEventType::UnauthorizedAccess,
            req.admin_username.as_deref().unwrap_or("unknown"),
            "127.0.0.1",
            &format!("Unauthorized attempt to update user {}: {}", username, e),
            false,
        );

        return Ok(HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("Unauthorized: Admin access required".to_string()),
        }));
    }

    // Load users from file
    let mut file_users = match auth::load_users(config::USERS_FILE_PATH) {
        Ok(users) => users,
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::UserModified,
                req.admin_username.as_deref().unwrap_or("admin"),
                "127.0.0.1",
                &format!("Failed to update user {}: {}", username, e),
                false,
            );

            return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error loading users: {}", e)),
            }));
        }
    };

    // Check if user exists and find their index
    let user_index = file_users.iter().position(|u| u.username == username);
    
    if user_index.is_none() {
        audit_logger.log_event(
            audit::AuditEventType::UserModified,
            "admin",
            "127.0.0.1",
            &format!("Failed to update user {}: user not found", username),
            false,
        );

        return Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("User not found".to_string()),
        }));
    }

    let idx = user_index.unwrap();
    let mut changes = vec![];
    
    // Update role if provided
    if let Some(role) = &req.role {
        file_users[idx].role = role.clone();
        changes.push(format!("role={}", role));
    }
    
    // Update email if provided
    if let Some(email) = &req.email {
        file_users[idx].email = Some(email.clone());
        changes.push(format!("email={}", email));
    }

    // Save updated users to file
    if let Err(e) = auth::save_users(&file_users, config::USERS_FILE_PATH) {
        audit_logger.log_event(
            audit::AuditEventType::UserModified,
            "admin",
            "127.0.0.1",
            &format!("Failed to update user {}: {}", username, e),
            false,
        );

        return Ok(HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some(format!("Error saving user: {}", e)),
        }));
    }

    audit_logger.log_event(
        audit::AuditEventType::UserModified,
        "admin",
        "127.0.0.1",
        &format!("Updated user {}: {}", username, changes.join(", ")),
        true,
    );

    let user = &file_users[idx];
    let user_response = UserResponse {
        username: user.username.clone(),
        role: user.role.clone(),
        email: user.email.clone(),
        created_at: user.created_at.clone(),
    };

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(user_response),
        error: None,
    }))
}

pub async fn delete_user(
    path: web::Path<String>,
    req: web::Json<DeleteUserRequest>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    // Verify admin role
    if let Err(e) = verify_admin_user(req.admin_username.as_deref()) {
        audit_logger.log_event(
            audit::AuditEventType::UnauthorizedAccess,
            req.admin_username.as_deref().unwrap_or("unknown"),
            "127.0.0.1",
            &format!("Unauthorized attempt to delete user {}: {}", username, e),
            false,
        );

        return Ok(HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("Unauthorized: Admin access required".to_string()),
        }));
    }

    // Prevent deletion of admin users
    if username.to_lowercase() == "admin" {
        audit_logger.log_event(
            audit::AuditEventType::UserDeleted,
            req.admin_username.as_deref().unwrap_or("admin"),
            "127.0.0.1",
            "Failed to delete user admin: cannot delete admin user",
            false,
        );

        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("Cannot delete admin user".to_string()),
        }));
    }

    // Load users from file
    let mut file_users = match auth::load_users(config::USERS_FILE_PATH) {
        Ok(users) => users,
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::UserDeleted,
                req.admin_username.as_deref().unwrap_or("admin"),
                "127.0.0.1",
                &format!("Failed to delete user {}: {}", username, e),
                false,
            );

            return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error loading users: {}", e)),
            }));
        }
    };

    // Find and remove the user
    let initial_len = file_users.len();
    file_users.retain(|u| u.username != username);

    if file_users.len() == initial_len {
        // User not found
        audit_logger.log_event(
            audit::AuditEventType::UserDeleted,
            "admin",
            "127.0.0.1",
            &format!("Failed to delete user {}: user not found", username),
            false,
        );

        return Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("User not found".to_string()),
        }));
    }

    // Save updated users to file
    if let Err(e) = auth::save_users(&file_users, config::USERS_FILE_PATH) {
        audit_logger.log_event(
            audit::AuditEventType::UserDeleted,
            "admin",
            "127.0.0.1",
            &format!("Failed to delete user {}: {}", username, e),
            false,
        );

        return Ok(HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some(format!("Error saving users: {}", e)),
        }));
    }

    audit_logger.log_event(
        audit::AuditEventType::UserDeleted,
        "admin",
        "127.0.0.1",
        &format!("Deleted user {}", username),
        true,
    );

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({"message": format!("User {} deleted", username)})),
        error: None,
    }))
}

pub async fn admin_change_password(
    path: web::Path<String>,
    req: web::Json<AdminChangePasswordRequest>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    // Verify admin role
    if let Err(e) = verify_admin_user(req.admin_username.as_deref()) {
        audit_logger.log_event(
            audit::AuditEventType::UnauthorizedAccess,
            req.admin_username.as_deref().unwrap_or("unknown"),
            "127.0.0.1",
            &format!("Unauthorized attempt to change password for {}: {}", username, e),
            false,
        );

        return Ok(HttpResponse::Forbidden().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("Unauthorized: Admin access required".to_string()),
        }));
    }

    if username != req.username {
        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some("Username mismatch".to_string()),
        }));
    }

    // Validate password strength
    if let Err(e) = auth::validate_password_strength(&req.new_password) {
        audit_logger.log_event(
            audit::AuditEventType::PasswordChange,
            req.admin_username.as_deref().unwrap_or("admin"),
            "127.0.0.1",
            &format!("Failed to reset password for {}: {}", username, e),
            false,
        );

        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<serde_json::Value>,
            error: Some(e),
        }));
    }

    // Load users from file
    let mut file_users = match auth::load_users(config::USERS_FILE_PATH) {
        Ok(users) => users,
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::PasswordChange,
                req.admin_username.as_deref().unwrap_or("admin"),
                "127.0.0.1",
                &format!("Failed to reset password for {}: {}", username, e),
                false,
            );

            return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error loading users: {}", e)),
            }));
        }
    };

    match auth::hash_password(&req.new_password) {
        Ok(password_hash) => {
            // Find and update the user's password
            if let Some(user) = file_users.iter_mut().find(|u| u.username == username) {
                user.password_hash = password_hash;

                // Save updated users to file
                if let Err(e) = auth::save_users(&file_users, config::USERS_FILE_PATH) {
                    audit_logger.log_event(
                        audit::AuditEventType::PasswordChange,
                        "admin",
                        "127.0.0.1",
                        &format!("Failed to reset password for {}: {}", username, e),
                        false,
                    );

                    return Ok(HttpResponse::InternalServerError().json(ApiResponse {
                        success: false,
                        data: None::<serde_json::Value>,
                        error: Some(format!("Error saving password: {}", e)),
                    }));
                }

                audit_logger.log_event(
                    audit::AuditEventType::PasswordChange,
                    "admin",
                    "127.0.0.1",
                    &format!("Admin reset password for user {}", username),
                    true,
                );

                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(serde_json::json!({"message": "Password reset successfully"})),
                    error: None,
                }))
            } else {
                audit_logger.log_event(
                    audit::AuditEventType::PasswordChange,
                    "admin",
                    "127.0.0.1",
                    &format!("Failed to reset password for {}: user not found", username),
                    false,
                );

                Ok(HttpResponse::NotFound().json(ApiResponse {
                    success: false,
                    data: None::<serde_json::Value>,
                    error: Some("User not found".to_string()),
                }))
            }
        }
        Err(e) => {
            audit_logger.log_event(
                audit::AuditEventType::PasswordChange,
                "admin",
                "127.0.0.1",
                &format!("Failed to reset password for {}: {}", username, e),
                false,
            );

            Ok(HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<serde_json::Value>,
                error: Some(format!("Error resetting password: {}", e)),
            }))
        }
    }
}

// Audit Log Endpoints

pub async fn get_audit_logs(
    query: web::Query<AuditLogQuery>,
    _users: web::Data<Vec<auth::User>>,
    audit_logger: web::Data<audit::AuditLogger>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(100).min(1000);

    let logs = if let Some(username) = &query.username {
        audit_logger.get_user_logs(username, limit)
    } else {
        audit_logger.get_logs(limit)
    };

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(logs),
        error: None,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/auth/login", web::post().to(login))
            .route("/auth/logout", web::post().to(logout))
            .route("/auth/current-user", web::get().to(get_current_user))
            .route("/auth/change-password", web::post().to(change_password))
            .route("/admin/users", web::get().to(list_users))
            .route("/admin/users", web::post().to(create_user))
            .route("/admin/users/{username}", web::put().to(update_user))
            .route("/admin/users/{username}", web::delete().to(delete_user))
            .route("/admin/users/{username}/password", web::post().to(admin_change_password))
            .route("/admin/audit-logs", web::get().to(get_audit_logs))
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
