// Audit logging for security events
#![allow(dead_code)]
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailure,
    LogoutSuccess,
    PasswordChange,
    PasswordReset,
    UserCreated,
    UserDeleted,
    UserModified,
    UnauthorizedAccess,
    SessionTimeout,
    PermissionDenied,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::LoginSuccess => write!(f, "LOGIN_SUCCESS"),
            AuditEventType::LoginFailure => write!(f, "LOGIN_FAILURE"),
            AuditEventType::LogoutSuccess => write!(f, "LOGOUT_SUCCESS"),
            AuditEventType::PasswordChange => write!(f, "PASSWORD_CHANGE"),
            AuditEventType::PasswordReset => write!(f, "PASSWORD_RESET"),
            AuditEventType::UserCreated => write!(f, "USER_CREATED"),
            AuditEventType::UserDeleted => write!(f, "USER_DELETED"),
            AuditEventType::UserModified => write!(f, "USER_MODIFIED"),
            AuditEventType::UnauthorizedAccess => write!(f, "UNAUTHORIZED_ACCESS"),
            AuditEventType::SessionTimeout => write!(f, "SESSION_TIMEOUT"),
            AuditEventType::PermissionDenied => write!(f, "PERMISSION_DENIED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub timestamp: String,
    pub event_type: String,
    pub username: String,
    pub ip_address: String,
    pub details: String,
    pub success: bool,
}

pub struct AuditLogger {
    logs: Arc<Mutex<VecDeque<AuditLog>>>,
    max_logs: usize,
}

impl AuditLogger {
    pub fn new(max_logs: usize) -> Self {
        AuditLogger {
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(max_logs))),
            max_logs,
        }
    }

    pub fn log_event(
        &self,
        event_type: AuditEventType,
        username: &str,
        ip_address: &str,
        details: &str,
        success: bool,
    ) {
        let log = AuditLog {
            timestamp: Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            username: username.to_string(),
            ip_address: ip_address.to_string(),
            details: details.to_string(),
            success,
        };

        if let Ok(mut logs) = self.logs.lock() {
            logs.push_back(log.clone());
            
            // Keep only the most recent logs
            while logs.len() > self.max_logs {
                logs.pop_front();
            }
        }

        // Also log to tracing
        if success {
            info!("AUDIT: {} - {} from {} - {}", event_type, username, ip_address, details);
        } else {
            warn!("AUDIT: {} - {} from {} - {}", event_type, username, ip_address, details);
        }
    }

    pub fn get_logs(&self, limit: usize) -> Vec<AuditLog> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_user_logs(&self, username: &str, limit: usize) -> Vec<AuditLog> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter()
                .rev()
                .filter(|log| log.username == username)
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear_logs(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }
}
