use std::error::Error;
use chrono::{ NaiveDateTime };
use sqlx::{
    FromRow,
    MySql,
};
use super::get_pool;

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct Lyrics {
    pub id: i32,
    pub username: String,
    pub post_time: NaiveDateTime,
    pub song: i32,
    pub kanji_content: String,
    pub romaji_content: String,
    pub english_content: String,
    pub comment: String,
}

#[allow(unused)]
#[derive(Debug, Default, Clone, FromRow)]
pub struct RecentLyricTranslation {
    pub song_slug: String,
    pub song_name: String,
    pub album_slug: String,
    pub band_slug: String,
    pub post_time: NaiveDateTime,
}

pub async fn get_recent_lyric_translations_by_band_id(band_id: i32) -> Result<Vec<RecentLyricTranslation>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, RecentLyricTranslation>(r#"
        SELECT lyrics.post_time, songs.song_slug, songs.song_name, albums.album_slug, bands.band_slug
        FROM lyrics
        JOIN songs ON lyrics.song = songs.id
        JOIN albums ON songs.album = albums.id
        JOIN bands ON songs.band = bands.id
        WHERE bands.id=? AND lyrics.is_deleted=0 AND songs.is_deleted=0
        ORDER BY lyrics.post_time DESC
        LIMIT 5;
    "#)
        .bind(band_id)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_lyrics_by_song_id(song_id: i32) -> Result<Vec<Lyrics>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Lyrics>(r#"
        SELECT * FROM lyrics
        WHERE song=? AND is_deleted=0
        ORDER BY lyrics.post_time ASC
        LIMIT 15;
    "#)
        .bind(song_id)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn get_lyrics_by_username_and_song_id(username: &str, song_id: i32) -> Result<Lyrics, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, Lyrics>(r#"
        SELECT * FROM lyrics
        WHERE username=? AND song=? AND is_deleted=0
        ORDER BY lyrics.post_time ASC
        LIMIT 15;
    "#)
        .bind(username)
        .bind(song_id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn create_lyrics(
    lyrics: Lyrics,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Lyrics>(r#"
        INSERT INTO lyrics (
            username, post_time, song, kanji_content, romaji_content, english_content, comment
        )
        VALUES (?, NOW(), ?, ?, ?, ?, ?)
    "#)
        .bind(&lyrics.username)
        .bind(&lyrics.song)
        .bind(lyrics.kanji_content)
        .bind(lyrics.romaji_content)
        .bind(lyrics.english_content)
        .bind(lyrics.comment)
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

pub async fn update_lyrics(
    lyrics: Lyrics,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Lyrics>(r#"
        UPDATE lyrics
        SET kanji_content=?, romaji_content=?, english_content=?, comment=?
        WHERE id=?
        LIMIT 1
    "#)
        .bind(lyrics.kanji_content)
        .bind(lyrics.romaji_content)
        .bind(lyrics.english_content)
        .bind(lyrics.comment)
        .bind(lyrics.id)
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

pub async fn mark_lyrics_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, Lyrics>(r#"
        UPDATE lyrics
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
