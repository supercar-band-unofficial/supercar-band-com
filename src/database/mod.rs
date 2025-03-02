use std::fs;
use serde::Deserialize;
use sqlx::{
    MySqlPool,
};
use tokio::sync::OnceCell;

#[derive(Deserialize)]
struct SecretsConfig {
    database: SecretsConfigDatabase,
}

#[derive(Deserialize)]
struct SecretsConfigDatabase {
    host: String,
    port: u16,
    user: String,
    password: String,
}

#[derive(Debug, PartialEq)]
pub enum QueryOrder {
    Asc,
    Desc,
}

pub static POOL: OnceCell<MySqlPool> = OnceCell::const_new();

pub async fn init_pool() {
    let secrets_toml = fs::read_to_string(format!(
        "{}/config/secrets.toml",
        env!("CARGO_MANIFEST_DIR")
    )).expect("Failed to read secrets.toml file.");
    let config: SecretsConfig = toml::from_str(&secrets_toml)
        .expect("Failed to parse secrets.toml file.");

    tracing::info!("Database host: {}", config.database.host);
    tracing::info!("Database port: {}", config.database.port);
    tracing::info!("Database user: {}", config.database.user);

    let database_url: &str = &format!(
        "mysql://{}:{}@{}:{}/supercar_band",
        config.database.user.as_str(), config.database.password.as_str(),
        config.database.host.as_str(), config.database.port,
    );

    let pool = MySqlPool::connect(database_url)
        .await
        .expect("Failed to create pool.");

    POOL.set(pool).expect("Pool already initialized.");
}

pub fn get_pool() -> &'static MySqlPool {
    POOL.get().expect("Pool is not initialized.")
}

pub mod albums;
pub use albums::Album;
pub use albums::AlbumSummary;
pub use albums::AlbumSearchResult;
pub use albums::AlbumType;
pub use albums::get_album_by_slug_and_band_id;
pub use albums::get_album_by_song_slug;
pub use albums::get_album_summaries_by_band_id;
pub use albums::get_albums_by_band_id;
pub use albums::find_albums_by_name;
pub use albums::get_lyrics_booklet_images;
pub use albums::create_album;
pub use albums::update_album;
pub use albums::mark_album_for_deletion;

pub mod bands;
pub use bands::Band;
pub use bands::get_all_bands;
pub use bands::get_band_by_id;
pub use bands::get_band_by_slug;
pub use bands::find_bands_by_name;
pub use bands::create_band;
pub use bands::update_band;
pub use bands::mark_band_for_deletion;

pub mod comments;
pub use comments::Comment;
pub use comments::CommentWithReplies;
pub use comments::CommentSectionName;
pub use comments::get_comments_count;
pub use comments::get_comment_by_id;
pub use comments::get_comments_in_range;
pub use comments::get_comments_in_range_with_replies;
pub use comments::create_comment;

pub mod initialize;

pub mod lyrics;
pub use lyrics::Lyrics;
pub use lyrics::RecentLyricTranslation;
pub use lyrics::get_lyrics_by_song_id;
pub use lyrics::get_lyrics_by_username_and_song_id;
pub use lyrics::get_recent_lyric_translations_by_band_id;
pub use lyrics::create_lyrics;
pub use lyrics::update_lyrics;
pub use lyrics::mark_lyrics_for_deletion;

pub mod photos;
pub use photos::Photo;
pub use photos::PhotoAlbum;
pub use photos::PhotoAlbumWithPreviews;
pub use photos::get_all_photo_albums;
pub use photos::get_photo_album_by_slug;
pub use photos::get_photo_album_by_title;
pub use photos::photo_album_by_id_is_empty;
pub use photos::get_photos_by_photo_album_slug;
pub use photos::get_photo_by_id;
pub use photos::create_photo_album;
pub use photos::update_photo_album;
pub use photos::mark_photo_album_for_deletion;
pub use photos::create_photo;
pub use photos::update_photo;
pub use photos::mark_photo_for_deletion;

pub mod site_events;
pub use site_events::SiteEvent;
pub use site_events::get_recent_site_events;
pub use site_events::notify_lyrics_created;
pub use site_events::notify_user_registered;
pub use site_events::notify_album_created;
pub use site_events::notify_band_created;
pub use site_events::notify_tabs_created;
pub use site_events::notify_photo_album_created;
pub use site_events::notify_photo_created;
pub use site_events::notify_video_category_created;
pub use site_events::notify_video_created;

pub mod songs;
pub use songs::JoinedSongSlugs;
pub use songs::Song;
pub use songs::SongSearchResult;
pub use songs::get_song_by_id;
pub use songs::get_all_song_slugs;
pub use songs::get_song_by_slug_and_band_id;
pub use songs::get_song_slugs_by_band_id;
pub use songs::get_song_slugs_by_ids;
pub use songs::find_songs_with_translations_by_name;
pub use songs::create_songs_by_names;

pub mod tabs;
pub use tabs::SongTab;
pub use tabs::SongTabType;
pub use tabs::JoinedSongTab;
pub use tabs::get_song_tab_by_id;
pub use tabs::get_song_tab_by_username_type_and_song_id;
pub use tabs::get_song_tabs_by_song_id;
pub use tabs::create_song_tab;
pub use tabs::update_song_tab;
pub use tabs::mark_tab_for_deletion;

pub mod users;
pub use users::User;
pub use users::UserGender;
pub use users::UserSummary;
pub use users::UserPermission;
pub use users::UserPermissionSet;
pub use users::UserPreference;
pub use users::UserPreferenceSet;
pub use users::get_user_by_username;
pub use users::get_users_count;
pub use users::get_users_in_range;
pub use users::create_user;
pub use users::update_user_at_login;
pub use users::update_user_profile_info;
pub use users::update_user_password;
pub use users::update_user_profile_picture;

pub mod videos;
pub use videos::Video;
pub use videos::VideoCategory;
pub use videos::VideoCategoryWithPreview;
pub use videos::get_all_video_categories;
pub use videos::get_video_category_by_slug;
pub use videos::get_video_category_by_id;
pub use videos::get_videos_by_video_category_slug;
pub use videos::video_category_by_id_is_empty;
pub use videos::get_video_by_id;
pub use videos::get_video_by_slug_and_category_id;
pub use videos::create_video_category;
pub use videos::update_video_category;
pub use videos::mark_video_category_for_deletion;
pub use videos::create_video;
pub use videos::update_video;
pub use videos::mark_video_for_deletion;
