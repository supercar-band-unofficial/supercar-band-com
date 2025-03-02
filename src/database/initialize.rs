use std::error::Error;
use sqlx::{
    FromRow,
    MySql,
};

use super::get_pool;

#[derive(Debug, Default, Clone, FromRow)]
struct IgnoreDataType {}

#[allow(unused)]
async fn create_albums_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS albums (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT now(),
            band INT(11) DEFAULT 0,
            album_name VARCHAR(100) DEFAULT '',
            album_type INT(11) DEFAULT 0,
            publisher VARCHAR(100) DEFAULT '',
            picture_url VARCHAR(320) DEFAULT '',
            song0 INT(11),
            song1 INT(11),
            song2 INT(11),
            song3 INT(11),
            song4 INT(11),
            song5 INT(11),
            song6 INT(11),
            song7 INT(11),
            song8 INT(11),
            song9 INT(11),
            song10 INT(11),
            song11 INT(11),
            song12 INT(11),
            song13 INT(11),
            song14 INT(11),
            song15 INT(11),
            song16 INT(11),
            song17 INT(11),
            song18 INT(11),
            song19 INT(11),
            song20 INT(11),
            song21 INT(11),
            song22 INT(11),
            song23 INT(11),
            song24 INT(11),
            song25 INT(11),
            song26 INT(11),
            song27 INT(11),
            song28 INT(11),
            song29 INT(11),
            song30 INT(11),
            song30 INT(11),
            song31 INT(11),
            song32 INT(11),
            song33 INT(11),
            song34 INT(11),
            song35 INT(11),
            song36 INT(11),
            song37 INT(11),
            song38 INT(11),
            song39 INT(11),
            release_day DATE,
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating albums table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_bands_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS bands (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            band_slug VARCHAR(600) DEFAULT '',
            band_name VARCHAR(600) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating bands table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_comments_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS comments (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            ip_address VARCHAR(50) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            section ENUM('home','lyrics','photos','videos','members','chatbox','tabs'),
            section_tag_id INT(11) DEFAULT -1,
            reply_id INT(11) DEFAULT -1,
            comment VARCHAR(5000) DEFAULT '',
            visibility INT(11) DEFAULT 0,
            likes INT(11) DEFAULT 0,
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating comments table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_lyrics_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS lyrics (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            song INT(11) DEFAULT 0,
            kanji_content MEDIUMTEXT DEFAULT '',
            romaji_content VARCHAR(4000) DEFAULT '',
            english_content VARCHAR(4000) DEFAULT '',
            comment VARCHAR(2000) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating lyrics table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_notifications_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS notifications (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            notification_type ENUM('direct_message', 'comment_reply', 'lyric_post_comment', 'tabs_comment', 'photo_comment', 'video_comment', 'profile_comment') DEFAULT 'direct_message',
            notifier_username VARCHAR(30) DEFAULT '',
            link VARCHAR(200) DEFAULT '',
            is_read BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating notifications table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_photos_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS photos (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            album INT(11) DEFAULT 0,
            title VARCHAR(100) DEFAULT '',
            description VARCHAR(1000) DEFAULT '',
            photo_filename VARCHAR(500) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating photos table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_photo_albums_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS photo_albums (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            title VARCHAR(100) DEFAULT '',
            description VARCHAR(1000) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating photo_albums table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_site_events_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS site_events (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            comment VARCHAR(1000) DEFAULT ''
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating site_events table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_songs_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS songs (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            band INT(11) DEFAULT 0,
            album INT(11) DEFAULT 0,
            song_name VARCHAR(600) DEFAULT '',
            tab_count INT(11) DEFAULT 0,
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating songs table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_tabs_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS tabs (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            song INT(11) DEFAULT 0,
            tab_type ENUM('lead_guitar','rhythm_guitar','bass_guitar','drums','keyboard') DEFAULT 'lead_guitar',
            tab_content MEDIUMTEXT DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating tabs table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_users_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS users (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(30) DEFAULT '',
            password VARCHAR(128) DEFAULT '',
            email VARCHAR(320) DEFAULT '',
            first_name VARCHAR(30) DEFAULT '',
            last_name VARCHAR(30) DEFAULT '',
            gender INT(11) DEFAULT 2,
            birthday DATE,
            about_me VARCHAR(4096) DEFAULT '',
            country VARCHAR(1024) DEFAULT '',
            profile_picture_filename VARCHAR(50) DEFAULT '',
            join_time DATETIME DEFAULT NOW(),
            last_login_time DATETIME DEFAULT NOW(),
            permissions SET('create_band', 'edit_band', 'delete_band', 'create_album', 'edit_album', 'delete_album', 'create_own_lyrics', 'edit_own_lyrics', 'edit_lyrics', 'delete_own_lyrics', 'delete_lyrics', 'create_own_tabs', 'edit_own_tabs', 'edit_tabs', 'delete_own_tabs', 'delete_tabs', 'create_own_photo_album', 'edit_own_photo_album', 'edit_photo_album', 'delete_own_photo_album', 'delete_photo_album', 'upload_own_photo', 'edit_own_photo', 'edit_photo', 'delete_own_photo', 'delete_photo', 'create_own_video_category', 'edit_own_video_category', 'edit_video_category', 'delete_own_video_category', 'delete_video_category', 'upload_own_video', 'edit_own_video', 'edit_video', 'delete_own_video', 'delete_video', 'create_own_comment', 'delete_own_comment', 'delete_comment', 'edit_own_profile_info', 'upload_own_profile_picture', 'send_dms', 'delete_user', 'approve_queued_deletion', 'undo_queued_deletion', 'ban_ips', 'edit_user_permissions')
                DEFAULT 'create_own_lyrics,edit_own_lyrics,delete_own_lyrics,create_own_tabs,edit_own_tabs,delete_own_tabs,create_own_photo_album,edit_own_photo_album,delete_own_photo_album,upload_own_photo,edit_own_photo,delete_own_photo,create_own_video_category,edit_own_video_category,delete_own_video_category,upload_own_video,edit_own_video,delete_own_video,create_own_comment,delete_own_comment,edit_own_profile_info,upload_own_profile_picture,send_dms',
            preferences SET('allow_profile_comments', 'allow_profile_guest_comments', 'allow_dms', 'notify_profile_comments', 'notify_dms', 'notify_comment_replies', 'notify_global_feed')
                DEFAULT 'allow_profile_comments,allow_profile_guest_comments,allow_dms,notify_profile_comments,notify_dms,notify_comment_replies,notify_global_feed',
            blocklist JSON,
            ip_address VARCHAR(50) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating users table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_videos_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS videos (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            slug VARCHAR(100) DEFAULT '',
            category INT(11) DEFAULT 0,
            title VARCHAR(100) DEFAULT '',
            video_url VARCHAR(1000) DEFAULT '',
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            description VARCHAR(1000) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating videos table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
async fn create_video_categories_table() -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        CREATE TABLE IF NOT EXISTS video_categories (
            id INT(11) AUTO_INCREMENT PRIMARY KEY,
            slug VARCHAR(100) DEFAULT '',
            title VARCHAR(100) DEFAULT '',
            username VARCHAR(30) DEFAULT '',
            post_time DATETIME DEFAULT NOW(),
            description VARCHAR(1000) DEFAULT '',
            is_deleted BOOLEAN DEFAULT 0
        ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
    "#)
        .fetch_optional(get_pool())
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Error creating video_categories table {:?}", error);
            Err(Box::new(error))
        }
    }
}

#[allow(unused)]
pub async fn create_all_tables() {
    create_albums_table().await;
    create_bands_table().await;
    create_comments_table().await;
    create_lyrics_table().await;
    create_photos_table().await;
    create_photo_albums_table().await;
    create_site_events_table().await;
    create_songs_table().await;
    create_tabs_table().await;
    create_users_table().await;
    create_videos_table().await;
    create_video_categories_table().await;
}
