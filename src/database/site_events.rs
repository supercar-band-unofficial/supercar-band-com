use askama_escape::{ escape, Html };
use chrono::NaiveDateTime;
use sqlx::{
    FromRow,
    MySql,
};
use urlencoding::encode;

use super::get_pool;
use crate::database::tabs::SongTabType;
use crate::util::format;
use crate::util::user::create_user_profile_href;

#[derive(Debug, FromRow)]
pub struct SiteEvent {
    pub comment: String,
    pub post_time: NaiveDateTime,
}

#[derive(Debug, Default, Clone, FromRow)]
struct IgnoreDataType {}

pub async fn get_recent_site_events() -> Result<Vec<SiteEvent>, Box<dyn std::error::Error>> {
    Ok(
        sqlx::query_as::<MySql, SiteEvent>("SELECT * FROM site_events ORDER BY post_time DESC LIMIT 5")
            .fetch_all(get_pool())
            .await?
    )
}

pub async fn notify_user_registered(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let member_account_href = create_user_profile_href(&username);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(format!(r#"<a href="{}">{}</a> created an account. Welcome to the site!"#, member_account_href, escaped_username))
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_lyrics_created(username: &str, band_slug: &str, album_slug: &str, song_slug: &str, song_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let escaped_song_name = escape(song_name, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let lyrics_href = format!("/lyrics/{}/{}/{}/?contributor={}", band_slug, album_slug, song_slug, encode(username));

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> created a translation of <a href="{}">「{}」</a> on the <a href="/lyrics/">Lyrics</a> page."#,
                member_account_href,
                escaped_username,
                lyrics_href,
                escaped_song_name,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_album_created(username: &str, band_slug: &str, album_slug: &str, album_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let escaped_album_name = escape(album_name, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let album_href = format!("/lyrics/{}/{}/", band_slug, album_slug);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> added the album <a href="{}">{}</a> to the <a href="/lyrics/">Lyrics</a> page."#,
                member_account_href,
                escaped_username,
                album_href,
                escaped_album_name,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_band_created(username: &str, band_slug: &str, band_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let escaped_band_name = escape(band_name, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let band_href = format!("/lyrics/{}/", band_slug);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> added the band <a href="{}">{}</a> to the <a href="/lyrics/">Lyrics</a> page."#,
                member_account_href,
                escaped_username,
                band_href,
                escaped_band_name,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_tabs_created(
    username: &str,
    band_slug: &str,
    song_slug: &str,
    song_name: &str,
    tab_type: &SongTabType,
) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let escaped_song_name = escape(song_name, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let tabs_href = format!("/tabs/{}/{}/{}/{}/", band_slug, song_slug, format::to_kebab_case(tab_type.to_string().as_str()), encode(username));

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> created <strong>{}</strong> tabs for <a href="{}">「{}」</a> on the <a href="/tabs/">Tabs</a> page."#,
                member_account_href,
                escaped_username,
                tab_type.as_display(),
                tabs_href,
                escaped_song_name,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_photo_album_created(username: &str, photo_album_slug: &str, photo_album_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let escaped_photo_album_title = escape(photo_album_title, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let photo_album_href = format!("/photos/{}/", photo_album_slug);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> added a photo album <a href="{}">{}</a> to the <a href="/photos/">Photos</a> page."#,
                member_account_href,
                escaped_username,
                photo_album_href,
                escaped_photo_album_title,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_photo_created(username: &str, photo_album_slug: &str, photo_id: i32, photo_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    if photo_title.is_empty() {
        return Ok(());
    }

    let escaped_username = escape(username, Html).to_string();
    let escaped_photo_title = escape(photo_title, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let photo_href = format!("/photos/{}/{}/", photo_album_slug, photo_id);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> added a photo <a href="{}">{}</a> to the <a href="/photos/">Photos</a> page."#,
                member_account_href,
                escaped_username,
                photo_href,
                escaped_photo_title,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_video_category_created(username: &str, video_category_slug: &str, video_category_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let escaped_username = escape(username, Html).to_string();
    let escaped_video_category_title = escape(video_category_title, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let video_category_href = format!("/videos/{}/", video_category_slug);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> added a video category <a href="{}">{}</a> to the <a href="/videos/">Videos</a> page."#,
                member_account_href,
                escaped_username,
                video_category_href,
                escaped_video_category_title,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}

pub async fn notify_video_created(username: &str, video_category_slug: &str, video_slug: &str, video_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    if video_title.is_empty() {
        return Ok(());
    }

    let escaped_username = escape(username, Html).to_string();
    let escaped_video_title = escape(video_title, Html).to_string();
    let member_account_href = create_user_profile_href(&username);
    let video_href = format!("/videos/{}/{}/", video_category_slug, video_slug);

    let _ = sqlx::query_as::<MySql, IgnoreDataType>(r#"
        INSERT INTO site_events (
            username, post_time, comment, privacy
        )
        VALUES (?, NOW(), ?, 1)
    "#)
        .bind(username)
        .bind(
            format!(
                r#"<a href="{}">{}</a> added a video <a href="{}">{}</a> to the <a href="/videos/">Videos</a> page."#,
                member_account_href,
                escaped_username,
                video_href,
                escaped_video_title,
            )
        )
        .fetch_optional(get_pool())
        .await;

    Ok(())
}
