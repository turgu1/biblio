// Role-based access control
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Librarian,
    User,
    Reader,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Librarian => write!(f, "librarian"),
            UserRole::User => write!(f, "user"),
            UserRole::Reader => write!(f, "reader"),
        }
    }
}

impl UserRole {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "librarian" => UserRole::Librarian,
            "user" => UserRole::User,
            _ => UserRole::Reader,
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_manage_libraries(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Librarian)
    }

    pub fn can_view_audit_logs(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_manage_permissions(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_browse_libraries(&self) -> bool {
        matches!(
            self,
            UserRole::Admin | UserRole::Librarian | UserRole::User | UserRole::Reader
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permission {
    pub name: String,
    pub description: String,
}

pub struct PermissionChecker;

impl PermissionChecker {
    pub fn has_permission(role: &UserRole, permission: &str) -> bool {
        match permission {
            // Admin permissions
            "manage_users" => role.can_manage_users(),
            "manage_libraries" => role.can_manage_libraries(),
            "view_audit_logs" => role.can_view_audit_logs(),
            "manage_permissions" => role.can_manage_permissions(),
            
            // General permissions
            "browse_libraries" => role.can_browse_libraries(),
            
            // Reader permissions
            "download_books" => true,
            "view_book_details" => true,
            
            _ => false,
        }
    }

    pub fn require_permission(role: &UserRole, permission: &str) -> Result<(), String> {
        if Self::has_permission(role, permission) {
            Ok(())
        } else {
            Err(format!("Permission denied: {}", permission))
        }
    }
}
