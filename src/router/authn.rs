/**
 * This file handles authenticating a user against the database,
 * and builds a user session in memory of data that needs to be readily
 * accessible on every request.
 */

use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use argon2::{
    password_hash::{
        rand_core::{ OsRng },
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use async_trait::async_trait;
use axum_login::{ AuthUser, AuthnBackend, UserId };
use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::database::{ self, User, UserPermissionSet, UserPreferenceSet };
use crate::util::geolocation::{ self, Geolocation };

#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub struct UserSession {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub ip_address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub permissions: UserPermissionSet,
    pub preferences: UserPreferenceSet,
}
impl AuthUser for UserSession {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

pub static USER_SESSIONS: OnceCell<RwLock<HashMap<i32, UserSession>>> = OnceCell::const_new();

pub async fn init_user_sessions() {
    USER_SESSIONS
        .set(RwLock::new(HashMap::new()))
        .expect("User sessions already initialized.");
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub ip_address: String,
}

#[derive(Clone, Default)]
pub struct Backend {
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = UserSession;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Credentials { username, password, ip_address }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = match async {
            let user = match database::get_user_by_username(&username).await {
                Ok(user) => user,
                Err(_) => User::default(),
            };

            let crate::database::User {
                id,
                username: stored_username,
                password: stored_password_hash,
                first_name,
                last_name,
                permissions,
                preferences,
                ..
            } = &user;

            let parsed_hash = PasswordHash::new(&stored_password_hash)?;
            Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;

            let location = match geolocation::find("99.174.217.40").await {
                Ok(location) => location,
                Err(_) => Geolocation::default(),
            };

            let user_session = Self::User {
                id: id.clone(),
                username: stored_username.clone(),
                password: stored_password_hash.clone(),
                first_name: first_name.clone(),
                last_name: last_name.clone(),
                ip_address,
                latitude: location.latitude,
                longitude: location.longitude,
                permissions: permissions.clone(),
                preferences: preferences.clone(),
            };

            let user_sessions = USER_SESSIONS.get().expect("User sessions not initialized.");
            let mut user_sessions_write = user_sessions.write().unwrap_or_else(|poisoned| poisoned.into_inner());
            user_sessions_write.insert(id.clone(), user_session.clone());

            Ok::<Self::User, Box<dyn Error>>(user_session)
        }.await {
            Ok(user) => Some(user),
            Err(_) => None,
        };

        Ok(user)
    }

    async fn get_user(
        &self,
        id: &UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user_sessions = USER_SESSIONS.get().expect("User sessions not initialized.");
        let user_sessions_read = user_sessions.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(
            user_sessions_read.get(&id).cloned()
        )
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;

pub fn create_password_hash(password: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(
        Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string()
    )
}

pub fn verify_password(stored_password_hash: &str, password_entry: &str,) -> Result<(), ()> {
    if let Ok(parsed_hash) = PasswordHash::new(&stored_password_hash) {
        match Argon2::default().verify_password(password_entry.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(()),
            Err(_) => Err(())
        }
    } else {
        Err(())
    }
}

pub fn update_user_session_ip(id: &i32, ip_address: &str) {
    let user_sessions = USER_SESSIONS.get().expect("User sessions not initialized.");
    let mut user_sessions_write = user_sessions.write().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(user_session) = user_sessions_write.get_mut(id) {
        user_session.ip_address = String::from(ip_address);
    }
}
