use std::path::Path;
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::SaltString;
use tracing::{debug, error, warn};
use std::fs;
use std::io::{self, BufRead};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: String, // admin, librarian, user, reader
    pub email: Option<String>,
    pub created_at: Option<String>,
}

/// Load users from the users.ids file
pub fn load_users(users_file_path: &str) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let path = Path::new(users_file_path);
    
    if !path.exists() {
        error!("Users file not found at {:?}", users_file_path);
        return Err("Users file not found".into());
    }

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut users = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse username:password_hash:role:email:created_at format
        // Note: created_at may contain colons (ISO 8601 with timezone), so we limit splits
        let parts: Vec<&str> = line.splitn(5, ':').collect();
        if parts.len() < 2 {
            warn!("Invalid format in users.ids at line {}: expected at least 'username:hash'", line_num + 1);
            continue;
        }

        // Extract fields, handling variable number of parts
        let username = parts[0].to_string();
        let password_hash = parts[1].to_string();
        let role = if parts.len() > 2 && !parts[2].is_empty() {
            parts[2].to_string()
        } else {
            "reader".to_string()
        };
        let email = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].to_string())
        } else {
            None
        };
        let created_at = if parts.len() > 4 && !parts[4].is_empty() {
            Some(parts[4].to_string())
        } else {
            None
        };

        users.push(User {
            username: username.clone(),
            password_hash,
            role,
            email,
            created_at,
        });

        debug!("Loaded user: {}", username);
    }

    if users.is_empty() {
        error!("No valid users found in {}", users_file_path);
        return Err("No users found in users file".into());
    }

    debug!("Loaded {} users from {}", users.len(), users_file_path);
    Ok(users)
}

/// Hash a password using Argon2
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, String> {
    use argon2::password_hash::rand_core::OsRng;
    
    let salt = SaltString::generate(&mut OsRng::default());
    let argon2 = Argon2::default();
    
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => Err(format!("Failed to hash password: {}", e)),
    }
}

/// Verify a password against a hash
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    match PasswordHash::new(password_hash) {
        Ok(parsed_hash) => {
            let argon2 = Argon2::default();
            match argon2.verify_password(password.as_bytes(), &parsed_hash) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        Err(e) => Err(format!("Invalid password hash format: {}", e)),
    }
}

/// Authenticate a user with username and password
pub fn authenticate_user(
    username: &str,
    password: &str,
    users: &[User],
) -> Result<bool, String> {
    let user = users.iter()
        .find(|u| u.username == username);

    match user {
        Some(user) => {
            match verify_password(password, &user.password_hash) {
                Ok(is_valid) => {
                    if is_valid {
                        debug!("User {} authenticated successfully", username);
                    } else {
                        warn!("Failed authentication attempt for user {}", username);
                    }
                    Ok(is_valid)
                }
                Err(e) => Err(format!("Password verification error: {}", e)),
            }
        }
        None => {
            warn!("Authentication attempt for non-existent user {}", username);
            Ok(false)
        }
    }
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

/// Save users to the users.ids file
pub fn save_users(users: &[User], users_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let mut file = fs::File::create(users_file_path)?;
    
    writeln!(file, "# Biblio users file - auto-generated")?;
    writeln!(file, "# Format: username:password_hash:role:email:created_at")?;
    writeln!(file)?;
    
    for user in users {
        let email = user.email.as_deref().unwrap_or("");
        let created_at = user.created_at.as_deref().unwrap_or("");
        writeln!(file, "{}:{}:{}:{}:{}", 
                 user.username, 
                 user.password_hash, 
                 user.role,
                 email,
                 created_at)?;
    }
    
    debug!("Saved {} users to {}", users.len(), users_file_path);
    Ok(())
}

/// Parse users from enhanced format with roles and metadata
pub fn load_users_with_metadata(users_file_path: &str) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let path = Path::new(users_file_path);
    
    if !path.exists() {
        error!("Users file not found at {:?}", users_file_path);
        return Err("Users file not found".into());
    }

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut users = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(':').collect();
        
        // Support both old format (username:hash) and new format (username:hash:role:email:created_at)
        if parts.len() < 2 {
            warn!("Invalid format in users.ids at line {}", line_num + 1);
            continue;
        }

        let username = parts[0].to_string();
        let password_hash = parts[1].to_string();
        let role = if parts.len() > 2 && !parts[2].is_empty() {
            parts[2].to_string()
        } else {
            "reader".to_string()
        };
        let email = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].to_string())
        } else {
            None
        };
        let created_at = if parts.len() > 4 && !parts[4].is_empty() {
            Some(parts[4].to_string())
        } else {
            None
        };

        users.push(User {
            username: username.clone(),
            password_hash,
            role,
            email,
            created_at,
        });

        debug!("Loaded user: {}", username);
    }

    if users.is_empty() {
        error!("No valid users found in {}", users_file_path);
        return Err("No users found in users file".into());
    }

    debug!("Loaded {} users from {}", users.len(), users_file_path);
    Ok(users)
}
