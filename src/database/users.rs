use std::error::Error;
use std::fmt;
use std::io;
use std::collections::HashSet;
use chrono::{ NaiveDateTime, NaiveDate };
use sqlx::{
    Decode,
    Encode,
    FromRow,
    MySql,
    Row,
    Type,
};
use strum_macros::{ EnumString, Display };

use super::QueryOrder;
use super::get_pool;
use crate::router::authn::create_password_hash;

/**
 * User Gender Enum
 */

#[derive(Debug, Default, Clone, Eq, PartialEq, Type)]
#[sqlx(type_name = "section")]
#[sqlx(rename_all = "lowercase")]
pub enum UserGender {
    Male,
    Female,
    #[default]
    Unknown,
}
impl From<i64> for UserGender {
    fn from(value: i64) -> UserGender {
        match value {
            1 => UserGender::Male,
            2 => UserGender::Female,
            _ => UserGender::Unknown,
        }
    }
}
impl From<UserGender> for i64 {
    fn from(value: UserGender) -> i64 {
        match value {
            UserGender::Male => 1,
            UserGender::Female => 2,
            UserGender::Unknown => 0,
        }
    }
}
impl fmt::Display for UserGender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            UserGender::Male => "Male",
            UserGender::Female => "Female",
            UserGender::Unknown => "Unknown",
        })
    }
}
impl From<&str> for UserGender {
    fn from(value: &str) -> UserGender {
        match value {
            "Male" => UserGender::Male,
            "Female" => UserGender::Female,
            _ => UserGender::Unknown,
        }
    }
}

/**
 * User Permission Enum
 */

#[allow(unused)]
#[derive(Clone, Debug, Default, Display, Hash, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum UserPermission {
    CreateBand,
    EditBand,
    DeleteBand,
    CreateAlbum,
    EditAlbum,
    DeleteAlbum,
    CreateOwnLyrics,
    EditOwnLyrics,
    EditLyrics,
    DeleteOwnLyrics,
    DeleteLyrics,
    CreateOwnTabs,
    EditOwnTabs,
    EditTabs,
    DeleteOwnTabs,
    DeleteTabs,
    CreateOwnPhotoAlbum,
    EditOwnPhotoAlbum,
    EditPhotoAlbum,
    DeleteOwnPhotoAlbum,
    DeletePhotoAlbum,
    UploadOwnPhoto,
    EditOwnPhoto,
    EditPhoto,
    DeleteOwnPhoto,
    DeletePhoto,
    CreateOwnVideoCategory,
    EditOwnVideoCategory,
    EditVideoCategory,
    DeleteOwnVideoCategory,
    DeleteVideoCategory,
    UploadOwnVideo,
    EditOwnVideo,
    EditVideo,
    DeleteOwnVideo,
    DeleteVideo,
    CreateOwnComment,
    DeleteOwnComment,
    DeleteComment,
    EditOwnProfileInfo,
    UploadOwnProfilePicture,
    SendDms,
    DeleteUser,
    ApproveQueuedDeletion,
    UndoQueuedDeletion,
    BanIps,
    EditUserPermissions,
    #[default]
    Unknown,
}

/**
 * HashSet of UserPermission enums, derived from sql SET.
 */

#[derive(Debug, Default, Clone)]
pub struct UserPermissionSet(HashSet<UserPermission>);
impl UserPermissionSet {
    // pub fn into_inner(self) -> HashSet<UserPermission> {
    //     self.0
    // }
    pub fn contains(&self, user_permission: &UserPermission) -> bool {
        self.0.contains(user_permission)
    }
}
impl<'r> Decode<'r, MySql> for UserPermissionSet {
    fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let string_value: &str = Decode::<MySql>::decode(value)?;
        let set: HashSet<UserPermission> = string_value
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<UserPermission>().unwrap_or_else(|_| UserPermission::Unknown))
            .collect();
        Ok(UserPermissionSet(set))
    }
}
impl<'q> Encode<'q, MySql> for UserPermissionSet {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<sqlx::encode::IsNull, Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>> {
        let joined = self.0.iter()
            .cloned()
            .map(|p| format!("{}", p))
            .collect::<Vec<String>>()
            .join(",");
        Encode::<MySql>::encode_by_ref(&joined, buf)
    }
}
impl Type<MySql> for UserPermissionSet {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }

    fn compatible(ty: &sqlx::mysql::MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty)
    }
}

/**
 * User Preference Enum
 */

#[allow(unused)]
#[derive(Clone, Debug, Default, Display, Hash, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum UserPreference {
    AllowProfileComments,
    AllowProfileGuestComments,
    AllowDms,
    NotifyProfileComments,
    NotifyDms,
    NotifyCommentReplies,
    NotifyGlobalFeed,
    #[default]
    Unknown,
}

/**
 * HashSet of UserPreference enums, derived from sql SET.
 */

#[derive(Debug, Default, Clone)]
pub struct UserPreferenceSet(HashSet<UserPreference>);
impl UserPreferenceSet {
    // pub fn into_inner(self) -> HashSet<UserPreference> {
    //     self.0
    // }
    pub fn contains(&self, user_preference: &UserPreference) -> bool {
        self.0.contains(user_preference)
    }
}
impl<'r> Decode<'r, MySql> for UserPreferenceSet {
    fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let string_value: &str = Decode::<MySql>::decode(value)?;
        let set: HashSet<UserPreference> = string_value
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<UserPreference>().unwrap_or_else(|_| UserPreference::Unknown))
            .collect();
        Ok(UserPreferenceSet(set))
    }
}
impl<'q> Encode<'q, MySql> for UserPreferenceSet {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<sqlx::encode::IsNull, Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>> {
        let joined = self.0.iter()
            .cloned()
            .map(|p| format!("{}", p))
            .collect::<Vec<String>>()
            .join(",");
        Encode::<MySql>::encode_by_ref(&joined, buf)
    }
}
impl Type<MySql> for UserPreferenceSet {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }

    fn compatible(ty: &sqlx::mysql::MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty)
    }
}

/**
 * Struct to represent a user profile
 */

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    #[sqlx(try_from = "i64")]
    pub gender: UserGender,
    pub birthday: Option<NaiveDate>,
    pub about_me: String,
    pub country: String,
    pub profile_picture_filename: String,
    pub join_time: NaiveDateTime,
    pub last_login_time: NaiveDateTime,
    pub ip_address: String,
    pub permissions: UserPermissionSet,
    pub preferences: UserPreferenceSet,
}

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct UserSummary {
    pub username: String,
    pub profile_picture_filename: String,
}

#[allow(unused)]
pub async fn get_user_by_id(
    id: &i32,
) -> Result<User, Box<dyn Error>> {
    Ok(
        sqlx::query_as::<MySql, User>(
            "SELECT * FROM users WHERE id=? AND is_deleted=0 LIMIT 1"
        )
            .bind(id)
            .fetch_one(get_pool())
            .await?
    )
}

pub async fn get_user_by_username(
    username: &str,
) -> Result<User, Box<dyn Error>> {
    if username.len() > 30 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Username was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, User>(
        "SELECT * FROM users WHERE LOWER(username)=? AND is_deleted=0 LIMIT 1"
    )
        .bind(username.to_lowercase())
        .fetch_one(get_pool())
        .await;

    match result {
        Ok(user) => {
            Ok(user)
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn get_users_count() -> Result<u32, Box<dyn Error>> {
    Ok(
        u32::try_from(sqlx::query("SELECT COUNT(*) FROM users WHERE is_deleted=0")
            .fetch_one(get_pool())
            .await?
            .get::<i64, usize>(0)
        )?
    )
}

pub async fn get_users_in_range(
    start: u32,
    length: u32,
    query_order: &QueryOrder,
) -> Result<Vec<UserSummary>, Box<dyn Error>> {
    let order = if query_order == &QueryOrder::Asc { "ASC" } else { "DESC" };
    Ok(
        sqlx::query_as::<MySql, UserSummary>(
            format!("SELECT username, profile_picture_filename FROM users WHERE is_deleted=0 ORDER BY id {} LIMIT ? OFFSET ?", order).as_str()
        )
            .bind(length)
            .bind(start)
            .fetch_all(get_pool())
            .await?
    )
}

pub async fn create_user(
    user: User,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let password_hash = create_password_hash(&user.password)?;

    let result = sqlx::query_as::<MySql, User>(r#"
        INSERT INTO users (
            username, password, email, first_name, last_name, gender,
            birthday, about_me, join_time, last_login_time, ip_address,
            blocklist, profile_picture_filename
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW(), ?, "[]", "Guest.jpeg")
    "#)
        .bind(&user.username)
        .bind(password_hash)
        .bind(user.email)
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(i64::from(user.gender))
        .bind(user.birthday)
        .bind(user.about_me)
        .bind(user.ip_address)
        .fetch_optional(get_pool())
        .await;

    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn update_user_at_login(
    username: &str,
    ip_address: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, User>(r#"
        UPDATE users
        SET last_login_time=NOW(), ip_address=?
        WHERE username=?
        LIMIT 1
    "#)
        .bind(ip_address)
        .bind(username)
        .fetch_optional(get_pool())
        .await;
    
    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn update_user_profile_info(
    user: User,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, User>(r#"
        UPDATE users
        SET first_name=?, last_name=?, email=?, gender=?, country=?, about_me=?
        WHERE username=?
        LIMIT 1
    "#)
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.email)
        .bind(i64::from(user.gender))
        .bind(user.country)
        .bind(user.about_me)
        .bind(user.username)
        .fetch_optional(get_pool())
        .await;
    
    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn update_user_password(
    username: &str,
    password: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let password_hash = create_password_hash(&password)?;

    let result = sqlx::query_as::<MySql, User>(r#"
        UPDATE users
        SET password=?
        WHERE username=?
        LIMIT 1
    "#)
        .bind(password_hash)
        .bind(username)
        .fetch_optional(get_pool())
        .await;
    
    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn update_user_profile_picture(
    username: &str,
    profile_picture_filename: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, User>(r#"
        UPDATE users
        SET profile_picture_filename=?
        WHERE username=?
        LIMIT 1
    "#)
        .bind(profile_picture_filename)
        .bind(username)
        .fetch_optional(get_pool())
        .await;
    
    match result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}
