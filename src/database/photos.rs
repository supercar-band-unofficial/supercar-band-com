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
pub struct PhotoAlbum {
    pub id: i32,
    pub username: String,
    pub post_time: NaiveDateTime,
    pub slug: String,
    pub title: String,
    pub description: String,
}

#[allow(unused)]
#[derive(Debug, Default, FromRow)]
pub struct Photo {
    pub id: i32,
    pub username: String,
    pub album: i32,
    pub post_time: NaiveDateTime,
    pub title: String,
    pub description: String,
    pub photo_filename: String,
}

#[allow(unused)]
#[derive(Debug, FromRow)]
pub struct PhotoAlbumWithPreviews {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub photo_filename_1: String,
    pub photo_filename_2: String,
    pub photo_filename_3: String,
}

#[allow(unused)]
#[derive(Debug, FromRow)]
pub struct InsertedPhoto {
    pub id: i32,
}

pub async fn get_all_photo_albums() -> Result<Vec<PhotoAlbumWithPreviews>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, PhotoAlbumWithPreviews>(r#"
        SELECT 
            photo_album.id,
            photo_album.slug,
            photo_album.title,
            photo_album.description,
            COALESCE((SELECT photo.photo_filename FROM photos photo WHERE photo.album = photo_album.id ORDER BY photo.id LIMIT 1 OFFSET 0), '') AS photo_filename_1,
            COALESCE((SELECT photo.photo_filename FROM photos photo WHERE photo.album = photo_album.id ORDER BY photo.id LIMIT 1 OFFSET 1), '') AS photo_filename_2,
            COALESCE((SELECT photo.photo_filename FROM photos photo WHERE photo.album = photo_album.id ORDER BY photo.id LIMIT 1 OFFSET 2), '') AS photo_filename_3
        FROM photo_albums photo_album
        WHERE photo_album.is_deleted=0
        LIMIT 1000
    "#)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_photo_album_by_slug(slug: &str) -> Result<PhotoAlbum, Box<dyn Error>> {
    if slug.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, PhotoAlbum>(r#"
        SELECT * from photo_albums
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

pub async fn photo_album_by_id_is_empty(id: i32) -> Result<(), Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Photo>(r#"
        SELECT * from photos
        WHERE album=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await;

    match result {
        Ok(_) => Err(Box::new(io::Error::new(io::ErrorKind::Other, "Photo found."))),
        Err(_) => Ok(())
    }
}

pub async fn get_photo_album_by_title(title: &str) -> Result<PhotoAlbum, Box<dyn Error>> {
    if title.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Title was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, PhotoAlbum>(r#"
        SELECT * from photo_albums
        WHERE title=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(title)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_photos_by_photo_album_slug(slug: &str) -> Result<Vec<Photo>, Box<dyn Error>> {
    if slug.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Photo>(r#"
        SELECT photos.* from photos
        JOIN photo_albums ON photo_albums.id = photos.album
        WHERE photo_albums.slug=? AND photos.is_deleted=0
        LIMIT 10000
    "#)
        .bind(slug)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_photo_by_id(id: i32) -> Result<Photo, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Photo>(r#"
        SELECT * from photos
        WHERE id=? AND photos.is_deleted=0
        LIMIT 1
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn create_photo_album(
    album: PhotoAlbum,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, PhotoAlbum>(r#"
        INSERT INTO photo_albums (
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

pub async fn update_photo_album(
    album: PhotoAlbum,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, PhotoAlbum>(r#"
        UPDATE photo_albums
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

pub async fn mark_photo_album_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, PhotoAlbum>(r#"
        UPDATE photo_albums
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

pub async fn create_photo(
    photo: Photo,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let pool = get_pool();

    let result = sqlx::query(r#"
        INSERT INTO photos (
            username, post_time, album, title, description, photo_filename
        )
        VALUES (?, NOW(), ?, ?, ?, ?)
    "#)
        .bind(photo.username)
        .bind(photo.album)
        .bind(photo.title)
        .bind(photo.description)
        .bind(photo.photo_filename)
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

pub async fn update_photo(
    photo: Photo,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Photo>(r#"
        UPDATE photos
        SET album=?, title=?, description=?, photo_filename=?
        WHERE id=?
        LIMIT 1
    "#)
        .bind(photo.album)
        .bind(photo.title)
        .bind(photo.description)
        .bind(photo.photo_filename)
        .bind(photo.id)
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

pub async fn mark_photo_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Photo>(r#"
        UPDATE photos
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