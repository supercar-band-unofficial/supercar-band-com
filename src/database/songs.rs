use std::error::Error;
use std::io;
use sqlx::{
    FromRow,
    MySql,
};
use super::get_pool;
use crate::util::format;
use crate::util::sql::sanitize_like_clause_value;

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct Song {
    pub id: i32,
    pub band: i32,
    pub album: i32,
    pub song_slug: String,
    pub song_name: String,
    pub tab_count: i32,
}

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct JoinedSongSlugs {
    pub song_name: String,
    pub song_slug: String,
    pub album_slug: String,
    pub band_slug: String,
    pub has_translation: bool,
    pub tab_count: i32,
}

pub async fn get_song_by_id(id: i32) -> Result<Song, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Song>(r#"
        SELECT * FROM songs
        WHERE id=? AND is_deleted=0
        LIMIT 1;
    "#)
        .bind(id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_song_by_slug_and_band_id(song_slug: &str, band_id: i32) -> Result<Song, Box<dyn Error>> {
    if song_slug.len() > 600 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Song slug was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, Song>(r#"
        SELECT * FROM songs
        WHERE song_slug=? AND band=? AND is_deleted=0
        LIMIT 1;
    "#)
        .bind(song_slug)
        .bind(band_id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_song_slugs_by_band_id(band_id: i32) -> Result<Vec<JoinedSongSlugs>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, JoinedSongSlugs>(r#"
        SELECT songs.song_slug, songs.song_name, albums.album_slug, bands.band_slug, songs.tab_count, false as has_translation
        FROM songs
        JOIN albums ON songs.album = albums.id
        JOIN bands ON songs.band = bands.id
        WHERE songs.band=? AND songs.is_deleted=0 AND albums.is_deleted=0 AND bands.is_deleted=0
        ORDER BY songs.song_slug
        LIMIT 10000;
    "#)
        .bind(band_id)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_all_song_slugs() -> Result<Vec<JoinedSongSlugs>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, JoinedSongSlugs>(r#"
        SELECT songs.song_slug, songs.song_name, albums.album_slug, bands.band_slug, songs.tab_count, false as has_translation
        FROM songs
        JOIN albums ON songs.album = albums.id
        JOIN bands ON songs.band = bands.id
        WHERE songs.is_deleted=0 AND albums.is_deleted=0 AND bands.is_deleted=0
        ORDER BY songs.album ASC, songs.band ASC
        LIMIT 10000;
    "#)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_song_slugs_by_ids(ids: &Vec<i32>) -> Result<Vec<JoinedSongSlugs>, Box<dyn Error>> {
    if ids.len() == 0 {
        return Ok(Vec::new());
    }

    let ids_group = ids
        .into_iter()
        .map(|i| format!("{}", i))
        .collect::<Vec<String>>()
        .join(", ");
        
    let query = format!(r#"
        SELECT
            songs.song_slug,
            songs.song_name,
            albums.album_slug,
            bands.band_slug,
            songs.tab_count,
            (SELECT EXISTS (
                SELECT 1 
                FROM lyrics 
                WHERE lyrics.song = songs.id AND lyrics.is_deleted=0
            )) AS has_translation
        FROM songs
        JOIN albums ON songs.album = albums.id
        JOIN bands ON songs.band = bands.id
        WHERE songs.id IN ({}) AND songs.is_deleted=0
        ORDER BY FIELD(songs.id, {})
        LIMIT 100;
    "#, ids_group, ids_group);

    let result = sqlx::query_as::<MySql, JoinedSongSlugs>(&query)
        .fetch_all(get_pool())
        .await?;

    Ok(result)
}

pub async fn create_songs_by_names(names: &Vec<&str>, album_id: i32, band_id: i32) -> Result<Vec<Option<i32>>, Box<dyn Error + Send + Sync>> {
    if names.len() == 0 {
        return Ok(Vec::new());
    }

    let mut ids: Vec<Option<i32>> = Vec::with_capacity(names.len());
    for name in names {
        if name.trim().is_empty() {
            ids.push(None);
            continue;
        }
        let song_slug = format::to_kebab_case(name);
        let song_result = sqlx::query_as::<MySql, Song>(r#"
            SELECT * from songs
            WHERE song_slug=? AND band=? AND is_deleted=0
        "#)
            .bind(&song_slug)
            .bind(band_id)
            .fetch_optional(get_pool())
            .await?;
        match song_result {
            Some(song) => {
                ids.push(Some(song.id));
            },
            None => {
                let _ = sqlx::query_as::<MySql, Song>(r#"
                    INSERT INTO songs (
                        band, album, song_slug, song_name, tab_count
                    )
                    VALUES (?, ?, ?, ?, 0)
                "#)
                    .bind(band_id)
                    .bind(album_id)
                    .bind(&song_slug)
                    .bind(name)
                    .fetch_optional(get_pool())
                    .await?;
                let insert_result = sqlx::query_as::<MySql, Song>(r#"
                    SELECT * from songs
                    WHERE song_slug=? AND band=? AND is_deleted=0
                "#)
                    .bind(&song_slug)
                    .bind(band_id)
                    .fetch_optional(get_pool())
                    .await?;
                match insert_result {
                    Some(song) => {
                        ids.push(Some(song.id));
                    },
                    None => {
                        ids.push(None);
                    }
                }
            },
        }
    }

    Ok(ids)
}

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct SongSearchResult {
    pub song_name: String,
    pub song_slug: String,
    pub album_slug: String,
    pub band_slug: String,
    pub band_name: String,
    pub has_translation: bool,
}

pub async fn find_songs_with_translations_by_name(search: &str) -> Result<Vec<SongSearchResult>, Box<dyn Error>> {
    if search.len() > 200 {
        return Err(
            Box::new(
                io::Error::new(io::ErrorKind::Other, "Search term was too long.")
            )
        );
    }

    let result = sqlx::query_as::<MySql, SongSearchResult>(r#"
        SELECT
            songs.song_slug,
            songs.song_name,
            albums.album_slug,
            bands.band_slug,
            bands.band_name,
            (SELECT EXISTS (
                SELECT 1 
                FROM lyrics 
                WHERE lyrics.song = songs.id AND lyrics.is_deleted=0
            )) AS has_translation
        FROM songs
        JOIN albums ON songs.album = albums.id
        JOIN bands ON songs.band = bands.id
        WHERE songs.song_name LIKE ? AND songs.is_deleted=0
        LIMIT 15;
    "#)
        .bind(format!("%{}%", sanitize_like_clause_value(search)))
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}
