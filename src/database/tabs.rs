use std::error::Error;
use chrono::NaiveDateTime;
use sqlx::{
    FromRow,
    MySql,
    Type,
};
use strum_macros::{ Display, EnumString };

use super::get_pool;
use crate::database::songs::Song;

#[derive(Clone, Debug, Default, Display, EnumString, Type)]
#[sqlx(type_name = "section")]
#[sqlx(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SongTabType {
    LeadGuitar,
    RhythmGuitar,
    BassGuitar,
    Drums,
    Keyboard,
    #[default]
    Unknown,
}
impl SongTabType {
    pub fn as_display(&self) -> &str {
        match self {
            SongTabType::LeadGuitar => "Lead Guitar",
            SongTabType::RhythmGuitar => "Rhythm Guitar",
            SongTabType::BassGuitar => "Bass Guitar",
            SongTabType::Drums => "Drums",
            SongTabType::Keyboard => "Keyboard",
            SongTabType::Unknown => "Unknown",
        }
    }
    pub fn to_values() -> Vec<SongTabType> {
        vec!(
            SongTabType::LeadGuitar,
            SongTabType::RhythmGuitar,
            SongTabType::BassGuitar,
            SongTabType::Drums,
            SongTabType::Keyboard,
        )
    }
}

#[allow(unused)]
#[derive(Clone, Debug, Default, FromRow)]
pub struct SongTab {
    pub id: i32,
    pub username: String,
    pub post_time: NaiveDateTime,
    pub song: i32,
    pub tab_type: SongTabType,
    pub tab_content: String,
}

pub async fn get_song_tab_by_id(id: i32) -> Result<SongTab, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, SongTab>(r#"
        SELECT * FROM tabs
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

pub async fn get_song_tab_by_username_type_and_song_id(username: &str, tab_type: &SongTabType, song_id: i32) -> Result<SongTab, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, SongTab>(r#"
        SELECT * FROM tabs
        WHERE username=? AND tab_type=? AND song=? AND is_deleted=0
        LIMIT 1;
    "#)
        .bind(username)
        .bind(tab_type)
        .bind(song_id)
        .fetch_one(get_pool())
        .await?;

    Ok(
        result
    )
}

#[allow(unused)]
#[derive(Clone, Debug, Default, FromRow)]
pub struct JoinedSongTab {
    pub username: String,
    pub song_slug: String,
    pub song_name: String,
    pub tab_type: SongTabType,
}

pub async fn get_song_tabs_by_song_id(id: i32) -> Result<Vec<JoinedSongTab>, Box<dyn Error>> {
    let result = sqlx::query_as::<MySql, JoinedSongTab>(r#"
        SELECT tabs.username, songs.song_slug, songs.song_name, tabs.tab_type
        FROM tabs
        JOIN songs ON tabs.song = songs.id
        WHERE tabs.song=? AND tabs.is_deleted=0 AND songs.is_deleted=0
        LIMIT 1000;
    "#)
        .bind(id)
        .fetch_all(get_pool())
        .await?;

    Ok(
        result
    )
}

pub async fn create_song_tab(
    tab: SongTab,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, SongTab>(r#"
        INSERT INTO tabs (
            username, post_time, song, tab_type, tab_content
        )
        VALUES (?, NOW(), ?, ?, ?)
    "#)
        .bind(tab.username)
        .bind(tab.song)
        .bind(tab.tab_type)
        .bind(tab.tab_content)
        .fetch_optional(get_pool())
        .await;

    let _ = sqlx::query_as::<MySql, Song>(r#"
        UPDATE songs
        SET tab_count=(SELECT COUNT(*) FROM tabs WHERE tabs.song = songs.id AND tabs.is_deleted=0)
        WHERE id=?
        LIMIT 1
    "#)
        .bind(tab.song)
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

pub async fn update_song_tab(
    tab: SongTab,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = sqlx::query_as::<MySql, SongTab>(r#"
        UPDATE tabs
        SET tab_type=?, tab_content=?
        WHERE id=?
        LIMIT 1
    "#)
        .bind(tab.tab_type)
        .bind(tab.tab_content)
        .bind(tab.id)
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

pub async fn mark_tab_for_deletion(
    id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let song_id = match get_song_tab_by_id(id).await {
        Ok(tab) => tab.song,
        Err(_) => -1,
    };

    let result = sqlx::query_as::<MySql, SongTab>(r#"
        UPDATE tabs
        SET is_deleted=1
        WHERE id=?
        LIMIT 1
    "#)
        .bind(id)
        .fetch_optional(get_pool())
        .await;
    
    let _ = sqlx::query_as::<MySql, Song>(r#"
        UPDATE songs
        SET tab_count=(SELECT COUNT(*) FROM tabs WHERE tabs.song = songs.id AND tabs.is_deleted=0)
        WHERE id=?
        LIMIT 1
    "#)
        .bind(song_id)
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
