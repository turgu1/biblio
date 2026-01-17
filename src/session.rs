// Session management for Biblio authentication
#![allow(dead_code)]
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub created_at: String,
    pub expires_at: String,
    pub last_activity: String,
}

pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    session_timeout_minutes: i64,
}

impl SessionStore {
    pub fn new(timeout_minutes: i64) -> Self {
        SessionStore {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_timeout_minutes: timeout_minutes,
        }
    }

    pub fn create_session(&self, username: &str) -> String {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.session_timeout_minutes);

        let session = Session {
            token: token.clone(),
            username: username.to_string(),
            created_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
            last_activity: now.to_rfc3339(),
        };

        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(token.clone(), session);
        }

        token
    }

    pub fn validate_session(&self, token: &str) -> Option<String> {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(token) {
                let now = Utc::now();
                
                // Check if session has expired
                if let Ok(expires_at) = DateTime::parse_from_rfc3339(&session.expires_at) {
                    if now > expires_at.with_timezone(&Utc) {
                        sessions.remove(token);
                        return None;
                    }
                }

                // Update last activity
                session.last_activity = now.to_rfc3339();
                return Some(session.username.clone());
            }
        }
        None
    }

    pub fn invalidate_session(&self, token: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(token);
        }
    }

    pub fn cleanup_expired_sessions(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            let now = Utc::now();
            sessions.retain(|_, session| {
                if let Ok(expires_at) = DateTime::parse_from_rfc3339(&session.expires_at) {
                    now <= expires_at.with_timezone(&Utc)
                } else {
                    false
                }
            });
        }
    }
}
