use std::io;
use std::error::Error;
use chrono::NaiveDateTime;
use sqlx::{
    FromRow,
    MySql,
};
use super::get_pool;

#[allow(unused)]
#[derive(Debug, Default, FromRow)]
pub struct VideoCategory {
    pub id: i32,
    pub username: String,
    pub post_time: NaiveDateTime,
    pub slug: String,
    pub title: String,
    pub description: String,
}

#[allow(unused)]
#[derive(Debug, Default, FromRow)]
pub struct Video {
    pub id: i32,
    pub slug: String,
    pub category: i32,
    pub title: String,
    pub video_url: String,
    pub username: String,
    pub post_time: NaiveDateTime,
    pub description: String,
}

#[allow(unused)]
#[derive(Debug, FromRow)]
pub struct VideoCategoryWithPreview {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub video_preview_url: String,
}

pub async fn get_all_video_categories() -> Result<Vec<VideoCategoryWithPreview>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, VideoCategoryWithPreview>(r#"
        SELECT 
            video_category.id,
            video_category.slug,
            video_category.title,
            COALESCE((SELECT video.video_url FROM videos video WHERE video.category = video_category.id ORDER BY video.id LIMIT 1 OFFSET 0), '') AS video_preview_url
        FROM video_categories video_category
        WHERE video_category.is_deleted=0
        LIMIT 1000
    "#)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_video_category_by_slug(slug: &str) -> Result<VideoCategory, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, VideoCategory>(r#"
        SELECT * from video_categories
        WHERE slug=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(slug)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_video_category_by_id(id: i32) -> Result<VideoCategory, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, VideoCategory>(r#"
        SELECT * from video_categories
        WHERE id=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn video_category_by_id_is_empty(id: i32) -> Result<(), Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Video>(r#"
        SELECT * from videos
        WHERE category=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await;

    match result {
        Ok(_) => Err(Box::new(io::Error::new(io::ErrorKind::Other, "Video found."))),
        Err(_) => Ok(())
    }
}

pub async fn get_videos_by_video_category_slug(slug: &str) -> Result<Vec<Video>, Box<dyn Error>> {
    if slug.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Video>(r#"
        SELECT videos.* from videos
        JOIN video_categories ON video_categories.id = videos.category
        WHERE video_categories.slug=? AND videos.is_deleted=0
        LIMIT 10000
    "#)
        .bind(slug)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_video_by_id(id: i32) -> Result<Video, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Video>(r#"
        SELECT * from videos
        WHERE id=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_video_by_slug_and_category_id(video_slug: &str, category_id: i32) -> Result<Video, Box<dyn Error>> {
    if video_slug.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Video>(r#"
        SELECT * from videos
        WHERE slug=? AND category=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(video_slug)
        .bind(category_id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn create_video_category(
    album: VideoCategory,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, VideoCategory>(r#"
        INSERT INTO video_categories (
            username, post_time, slug, title, description
        )
        VALUES (?, NOW(), ?, ?, ?)
    "#)
        .bind(album.username)
        .bind(album.slug)
        .bind(album.title)
        .bind(album.description)
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

pub async fn update_video_category(
    album: VideoCategory,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, VideoCategory>(r#"
        UPDATE video_categories
        SET slug=?, title=?, description=?
        WHERE id=?
        LIMIT 1
    "#)
        .bind(album.slug)
        .bind(album.title)
        .bind(album.description)
        .bind(album.id)
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

pub async fn mark_video_category_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, VideoCategory>(r#"
        UPDATE video_categories
        SET is_deleted=1
        WHERE id=?
        LIMIT 1
    "#)
        .bind(id)
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

pub async fn create_video(
    video: Video,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let pool = get_pool();

    let result = sqlx::query(r#"
        INSERT INTO videos (
            username, post_time, category, slug, title, description, video_url
        )
        VALUES (?, NOW(), ?, ?, ?, ?, ?)
    "#)
        .bind(video.username)
        .bind(video.category)
        .bind(video.slug)
        .bind(video.title)
        .bind(video.description)
        .bind(video.video_url)
        .execute(pool)
        .await;
    
    match result {
        Ok(result) => {
            Ok(i32::try_from(result.last_insert_id()).unwrap_or_else(|_| 999))
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

pub async fn update_video(
    video: Video,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Video>(r#"
        UPDATE videos
        SET category=?, slug=?, title=?, description=?, video_url=?
        WHERE id=?
        LIMIT 1
    "#)
        .bind(video.category)
        .bind(video.slug)
        .bind(video.title)
        .bind(video.description)
        .bind(video.video_url)
        .bind(video.id)
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

pub async fn mark_video_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Video>(r#"
        UPDATE videos
        SET is_deleted=1
        WHERE id=?
        LIMIT 1
    "#)
        .bind(id)
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