/**
 * Tracking for password reset sessions.
 */

use chrono::prelude::{ DateTime, Utc };
use dashmap::DashMap;
use tokio::sync::OnceCell;
use uuid::Uuid;

pub static PASSWORD_RESET_SESSIONS: OnceCell<DashMap<String, PasswordResetSession>> = OnceCell::const_new();
pub static SESSION_EXPIRY_SECONDS: i64 = 1800;

#[derive(Clone, Debug)]
pub struct PasswordResetSession {
    pub username: String,
    pub visited: bool,
    pub timestamp: DateTime<Utc>
}

pub fn init_password_reset_sessions() {
    PASSWORD_RESET_SESSIONS
        .set(DashMap::new())
        .expect("Password reset sessions already initialized.");
}

pub fn create_password_reset_session(username: String) -> String {
    let password_reset_sessions = PASSWORD_RESET_SESSIONS.get().expect("Password reset sessions not initialized.");
    let session_id = format!("{}", Uuid::new_v4());
    
    password_reset_sessions.insert(
        session_id.clone(),
        PasswordResetSession {
            username,
            visited: false,
            timestamp: Utc::now(),
        }
    );

    let now = Utc::now();
    password_reset_sessions.retain(|_, session| {
        now.signed_duration_since(session.timestamp).num_seconds() < SESSION_EXPIRY_SECONDS
    });

    session_id
}

pub fn get_password_reset_session_username(session_id: &str) -> Option<String> {
    let password_reset_sessions = PASSWORD_RESET_SESSIONS.get().expect("Password reset sessions not initialized.");
    let now = Utc::now();

    match password_reset_sessions.get_mut(session_id) {
        Some(mut session) => {
            if now.signed_duration_since(session.timestamp).num_seconds() < SESSION_EXPIRY_SECONDS {
                if !session.visited {
                    session.visited = true;
                    session.timestamp = Utc::now();
                }
                return Some(session.username.clone());
            }
            password_reset_sessions.remove(session_id);
            None
        },
        None => None,
    }
}

pub fn discard_password_reset_session(session_id: &str) {
    let password_reset_sessions = PASSWORD_RESET_SESSIONS.get().expect("Password reset sessions not initialized.");
    password_reset_sessions.remove(session_id);
}
