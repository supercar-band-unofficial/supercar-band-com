use std::fmt;
use std::error::Error;
use std::io;
use chrono::{ NaiveDateTime, NaiveDate };
use regex::Regex;
use sqlx::{
    FromRow,
    MySql,
    Type,
};

use super::get_pool;
use crate::database::bands;
use crate::database::songs::Song;
use crate::util::filesystem;
use crate::util::sql::sanitize_like_clause_value;

pub static LYRICS_BOOKLET_BASE_DIRECTORY: &str = "uploads/assets/images/booklets";
pub static LYRICS_BOOKLET_BASE_URL: &str = "/assets/images/booklets";

#[derive(Debug, Default, Clone, Eq, PartialEq, Type)]
#[sqlx(type_name = "section")]
#[sqlx(rename_all = "lowercase")]
pub enum AlbumType {
    Full,
    Single,
    Compilation,
    Remix,
    #[default]
    Unknown,
}
impl AlbumType {
    pub fn as_key(&self) -> &str {
        match self {
            AlbumType::Full => "full",
            AlbumType::Single => "single",
            AlbumType::Compilation => "compilation",
            AlbumType::Remix => "remix",
            AlbumType::Unknown => "unknown",
        }
    }
    pub fn to_values() -> Vec<AlbumType> {
        vec!(
            AlbumType::Full,
            AlbumType::Single,
            AlbumType::Compilation,
            AlbumType::Remix,
        )
    }
}
impl fmt::Display for AlbumType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            AlbumType::Full => "Full Album",
            AlbumType::Single => "Single Album",
            AlbumType::Compilation => "Compilation Album",
            AlbumType::Remix => "Remix Album",
            _ => "Unknown",
        })
    }
}
impl From<i64> for AlbumType {
    fn from(value: i64) -> AlbumType {
        match value {
            0 => AlbumType::Full,
            1 => AlbumType::Single,
            2 => AlbumType::Compilation,
            3 => AlbumType::Remix,
            _ => AlbumType::Unknown,
        }
    }
}
impl From<AlbumType> for i64 {
    fn from(value: AlbumType) -> i64 {
        match value {
            AlbumType::Full => 0,
            AlbumType::Single => 1,
            AlbumType::Compilation => 2,
            AlbumType::Remix => 3,
            AlbumType::Unknown => -1,
        }
    }
}
impl From<&str> for AlbumType {
    fn from(value: &str) -> AlbumType {
        match value {
            "full" => AlbumType::Full,
            "single" => AlbumType::Single,
            "compilation" => AlbumType::Compilation,
            "remix" => AlbumType::Remix,
            _ => AlbumType::Unknown,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct Album {
    pub id: i32,
    pub username: String,
    pub post_time: NaiveDateTime,
    pub band: i32,
    pub album_slug: String,
    pub album_name: String,
    #[sqlx(try_from = "i64")]
    pub album_type: AlbumType,
    pub publisher: String,
    pub cover_picture_filename: String,
    pub song0: Option<i32>,
    pub song1: Option<i32>,
    pub song2: Option<i32>,
    pub song3: Option<i32>,
    pub song4: Option<i32>,
    pub song5: Option<i32>,
    pub song6: Option<i32>,
    pub song7: Option<i32>,
    pub song8: Option<i32>,
    pub song9: Option<i32>,
    pub song10: Option<i32>,
    pub song11: Option<i32>,
    pub song12: Option<i32>,
    pub song13: Option<i32>,
    pub song14: Option<i32>,
    pub song15: Option<i32>,
    pub song16: Option<i32>,
    pub song17: Option<i32>,
    pub song18: Option<i32>,
    pub song19: Option<i32>,
    pub song20: Option<i32>,
    pub song21: Option<i32>,
    pub song22: Option<i32>,
    pub song23: Option<i32>,
    pub song24: Option<i32>,
    pub song25: Option<i32>,
    pub song26: Option<i32>,
    pub song27: Option<i32>,
    pub song28: Option<i32>,
    pub song29: Option<i32>,
    pub song30: Option<i32>,
    pub song31: Option<i32>,
    pub song32: Option<i32>,
    pub song33: Option<i32>,
    pub song34: Option<i32>,
    pub song35: Option<i32>,
    pub song36: Option<i32>,
    pub song37: Option<i32>,
    pub song38: Option<i32>,
    pub song39: Option<i32>,
    pub release_day: NaiveDate,
}
impl Album {
    pub fn song_ids(&self) -> Vec<i32> {
        vec![
            self.song0, self.song1, self.song2, self.song3, self.song4, self.song5, self.song6, self.song7, self.song8, self.song9,
            self.song10, self.song11, self.song12, self.song13, self.song14, self.song15, self.song16, self.song17, self.song18, self.song19,
            self.song20, self.song21, self.song22, self.song23, self.song24, self.song25, self.song26, self.song27, self.song28, self.song29,
            self.song30, self.song31, self.song32, self.song33, self.song34, self.song35, self.song36, self.song37, self.song38, self.song39,
        ]
            .into_iter()
            .filter_map(|value| match value {
                Some(value) if value != 0 => Some(value),
                _ => None,
            })
            .collect()
    }
    pub fn populate_song_ids(mut self, songs: Vec<Option<i32>>) -> Self {
        let none: &Option<i32> = &None;
        self.song0 = *songs.get(0).unwrap_or_else(|| none);
        self.song1 = *songs.get(1).unwrap_or_else(|| none);
        self.song2 = *songs.get(2).unwrap_or_else(|| none);
        self.song3 = *songs.get(3).unwrap_or_else(|| none);
        self.song4 = *songs.get(4).unwrap_or_else(|| none);
        self.song5 = *songs.get(5).unwrap_or_else(|| none);
        self.song6 = *songs.get(6).unwrap_or_else(|| none);
        self.song7 = *songs.get(7).unwrap_or_else(|| none);
        self.song8 = *songs.get(8).unwrap_or_else(|| none);
        self.song9 = *songs.get(9).unwrap_or_else(|| none);
        self.song10 = *songs.get(10).unwrap_or_else(|| none);
        self.song11 = *songs.get(11).unwrap_or_else(|| none);
        self.song12 = *songs.get(12).unwrap_or_else(|| none);
        self.song13 = *songs.get(13).unwrap_or_else(|| none);
        self.song14 = *songs.get(14).unwrap_or_else(|| none);
        self.song15 = *songs.get(15).unwrap_or_else(|| none);
        self.song16 = *songs.get(16).unwrap_or_else(|| none);
        self.song17 = *songs.get(17).unwrap_or_else(|| none);
        self.song18 = *songs.get(18).unwrap_or_else(|| none);
        self.song19 = *songs.get(19).unwrap_or_else(|| none);
        self.song20 = *songs.get(20).unwrap_or_else(|| none);
        self.song21 = *songs.get(21).unwrap_or_else(|| none);
        self.song22 = *songs.get(22).unwrap_or_else(|| none);
        self.song23 = *songs.get(23).unwrap_or_else(|| none);
        self.song24 = *songs.get(24).unwrap_or_else(|| none);
        self.song25 = *songs.get(25).unwrap_or_else(|| none);
        self.song26 = *songs.get(26).unwrap_or_else(|| none);
        self.song27 = *songs.get(27).unwrap_or_else(|| none);
        self.song28 = *songs.get(28).unwrap_or_else(|| none);
        self.song29 = *songs.get(29).unwrap_or_else(|| none);
        self.song30 = *songs.get(30).unwrap_or_else(|| none);
        self.song31 = *songs.get(31).unwrap_or_else(|| none);
        self.song32 = *songs.get(32).unwrap_or_else(|| none);
        self.song33 = *songs.get(33).unwrap_or_else(|| none);
        self.song34 = *songs.get(34).unwrap_or_else(|| none);
        self.song35 = *songs.get(35).unwrap_or_else(|| none);
        self.song36 = *songs.get(36).unwrap_or_else(|| none);
        self.song37 = *songs.get(37).unwrap_or_else(|| none);
        self.song38 = *songs.get(38).unwrap_or_else(|| none);
        self.song39 = *songs.get(39).unwrap_or_else(|| none);
        self
    }
}

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct AlbumSummary {
    pub id: i32,
    pub username: String,
    pub band: i32,
    pub album_slug: String,
    pub album_name: String,
    #[sqlx(try_from = "i64")]
    pub album_type: AlbumType,
    pub cover_picture_filename: String,
}

pub async fn get_album_by_slug_and_band_id(album_slug: &str, band_id: i32) -> Result<Album, Box<dyn Error>> {
    if album_slug.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Album slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Album>(r#"
        SELECT * FROM albums
        WHERE album_slug=? AND band=? AND is_deleted=0
        LIMIT 1;
    "#)
        .bind(album_slug)
        .bind(band_id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

#[allow(unused)]
pub async fn get_album_by_slug_and_band_slug(album_slug: &str, band_slug: &str) -> Result<Album, Box<dyn Error>> {
    if band_slug.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Band slug was too long.")
            )
        );
    }
    let band_id = if let Ok(band) = bands::get_band_by_slug(band_slug).await {
        band.id
    } else {
        0
    };
    get_album_by_slug_and_band_id(album_slug, band_id).await
}

pub async fn get_album_by_song_slug(song_slug: &str, band_slug: &str) -> Result<Album, Box<dyn Error>> {
    if song_slug.len() > 600 || band_slug.len() > 600 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Song or band slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Album>(r#"
        SELECT albums.*, songs.song_slug, bands.band_slug
        FROM albums
        JOIN songs ON songs.album = albums.id
        JOIN bands ON songs.band = bands.id
        WHERE songs.song_slug=? AND bands.band_slug=?
            AND bands.is_deleted=0 AND albums.is_deleted=0
        LIMIT 1;
    "#)
        .bind(song_slug)
        .bind(band_slug)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_album_summaries_by_band_id(id: i32) -> Result<Vec<AlbumSummary>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, AlbumSummary>(r#"
        SELECT id, username, band, album_slug, album_name, album_type, cover_picture_filename FROM albums
        WHERE band=? AND is_deleted=0
        ORDER BY album_type ASC, release_day ASC
        LIMIT 1000;
    "#)
        .bind(id)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_albums_by_band_id(id: i32) -> Result<Vec<Album>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Album>(r#"
        SELECT * FROM albums
        WHERE band=? AND is_deleted=0
        ORDER BY album_type ASC, release_day ASC
        LIMIT 1000;
    "#)
        .bind(id)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}


#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct AlbumSearchResult {
    pub album_slug: String,
    pub album_name: String,
    pub band_slug: String,
    pub band_name: String,
}

pub async fn find_albums_by_name(search: &str) -> Result<Vec<AlbumSearchResult>, Box<dyn Error>> {
    if search.len() > 100 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Search term was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, AlbumSearchResult>(r#"
        SELECT
            albums.album_slug,
            albums.album_name,
            bands.band_slug,
            bands.band_name
        FROM albums
        JOIN bands ON albums.band = bands.id
        WHERE albums.album_name LIKE ? AND albums.is_deleted=0
        LIMIT 15;
    "#)
        .bind(format!("%{}%", sanitize_like_clause_value(search)))
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_lyrics_booklet_images(
    band_slug: &str,
    album_slug: &str,
) -> Vec<String> {
    let mut images: Vec<String> = Vec::new();
    let path = filesystem::get_filesystem_path(LYRICS_BOOKLET_BASE_DIRECTORY).await.join(
        format!("{}/{}", band_slug, album_slug)
    );
    let album_image_regex = Regex::new(r"^[0-9]{2}(-.*)?\.(jpeg|jpg|png|gif|webp|avif)$").unwrap();
    if let Ok(mut directory) = tokio::fs::read_dir(path).await {
        loop {
            match directory.next_entry().await {
                Ok(entry_option) => {
                    match entry_option {
                        Some(entry) => {
                            if let Ok(file_name) = entry.file_name().into_string() {
                                if album_image_regex.is_match(&file_name) {
                                    images.push(
                                        format!(
                                            "{}/{}/{}/{}",
                                            LYRICS_BOOKLET_BASE_URL,
                                            band_slug,
                                            album_slug,
                                            file_name,
                                        )
                                    );
                                }
                            }
                        },
                        None => {
                            break;
                        }
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
    }
    images.sort();
    images
}

pub async fn create_album(
    album: Album,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Album>(r#"
        INSERT INTO albums (
            username, post_time, band, album_slug, album_name, album_type, publisher, cover_picture_filename, release_day
        )
        VALUES (?, NOW(), ?, ?, ?, ?, ?, ?, ?)
    "#)
        .bind(album.username)
        .bind(album.band)
        .bind(album.album_slug)
        .bind(album.album_name)
        .bind(i64::from(album.album_type))
        .bind(album.publisher)
        .bind(album.cover_picture_filename)
        .bind(album.release_day)
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

pub async fn update_album(
    album: Album
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Album>(r#"
        UPDATE albums
        SET album_slug=?, album_name=?, album_type=?, publisher=?, cover_picture_filename=?, release_day=?,
            song0=?, song1=?, song2=?, song3=?, song4=?, song5=?, song6=?, song7=?, song8=?, song9=?,
            song10=?, song11=?, song12=?, song13=?, song14=?, song15=?, song16=?, song17=?, song18=?, song19=?,
            song20=?, song21=?, song22=?, song23=?, song24=?, song25=?, song26=?, song27=?, song28=?, song29=?,
            song30=?, song31=?, song32=?, song33=?, song34=?, song35=?, song36=?, song37=?, song38=?, song39=?
        WHERE id=? AND is_deleted=0
        LIMIT 1
    "#)
        .bind(album.album_slug)
        .bind(album.album_name)
        .bind(i64::from(album.album_type))
        .bind(album.publisher)
        .bind(album.cover_picture_filename)
        .bind(album.release_day)
        .bind(album.song0).bind(album.song1).bind(album.song2).bind(album.song3).bind(album.song4).bind(album.song5).bind(album.song6).bind(album.song7).bind(album.song8).bind(album.song9)
        .bind(album.song10).bind(album.song11).bind(album.song12).bind(album.song13).bind(album.song14).bind(album.song15).bind(album.song16).bind(album.song17).bind(album.song18).bind(album.song19)
        .bind(album.song20).bind(album.song21).bind(album.song22).bind(album.song23).bind(album.song24).bind(album.song25).bind(album.song26).bind(album.song27).bind(album.song28).bind(album.song29)
        .bind(album.song30).bind(album.song31).bind(album.song32).bind(album.song33).bind(album.song34).bind(album.song35).bind(album.song36).bind(album.song37).bind(album.song38).bind(album.song39)
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

pub async fn mark_album_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let album = sqlx::query_as::<MySql, Album>(r#"
        SELECT * FROM albums
        WHERE id=?
        LIMIT 1;
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await?;

    let delete_result = sqlx::query_as::<MySql, Album>(r#"
        UPDATE albums
        SET is_deleted=1
        WHERE id=?
        LIMIT 1
    "#)
        .bind(id)
        .fetch_optional(get_pool())
        .await;

    match delete_result {
        Ok(_) => {
            let song_ids = album.song_ids();
            let song_ids_by_album = sqlx::query_as::<MySql, Album>(r#"
                SELECT * from albums
                WHERE is_deleted=0
                LIMIT 100;
            "#)
                .fetch_all(get_pool())
                .await?
                .into_iter()
                .map(|album| album.song_ids())
                .collect::<Vec<_>>();
            for song_id in &song_ids {
                let mut is_song_referenced: bool = false;
                for other_album_song_ids in &song_ids_by_album {
                    if other_album_song_ids.iter().find(|check_song_id| &song_id == check_song_id).is_some() {
                        is_song_referenced = true;
                        break;
                    }
                }

                if !is_song_referenced {
                    let _ = sqlx::query_as::<MySql, Song>(r#"
                        UPDATE songs
                        SET is_deleted=1
                        WHERE id=?
                        LIMIT 1
                    "#)
                        .bind(song_id)
                        .fetch_optional(get_pool())
                        .await;
                }
            }

            Ok(())
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}
