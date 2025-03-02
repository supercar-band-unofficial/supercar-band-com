use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ Lyrics, Song };
use crate::util::user::create_user_profile_href;

pub struct SongLyricsParams {
    pub album_name: String,
    pub album_slug: String,
    pub band_name: String,
    pub band_slug: String,
    pub song: Song,
    pub lyrics: Option<Lyrics>,
}

pub struct CombinedLyricsLine {
    pub kanji: String,
    pub romaji: String,
    pub english: String,
}

#[derive(Template)]
#[template(path = "ui_modules/song_lyrics.html")]
pub struct SongLyricsTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    pub album_name: String,
    pub album_slug: String,
    band_name: String,
    band_slug: String,
    pub song: Song,
    lyrics: Option<Lyrics>,
    combined_lyrics: Vec<CombinedLyricsLine>,
}
impl<'a> SongLyricsTemplate<'a> {
    pub async fn new(
        params: SongLyricsParams,
    ) -> Result<SongLyricsTemplate<'a>, Box<dyn Error>> {
        let SongLyricsParams { album_name, album_slug, band_name, band_slug, lyrics, song } = params;

        let mut combined_lyrics: Vec<CombinedLyricsLine> = Vec::new();
        if let Some(lyrics) = &lyrics {
            let kanji_lines = lyrics.kanji_content.lines();
            let romaji_lines = lyrics.romaji_content.lines();
            let english_lines = lyrics.english_content.lines();
            for (kanji, (romaji, english)) in kanji_lines.zip(romaji_lines.zip(english_lines)) {
                combined_lyrics.push(CombinedLyricsLine {
                    kanji: kanji.to_string(),
                    romaji: romaji.to_string(),
                    english: english.to_string(),
                })
            }
        }

        Ok(SongLyricsTemplate {
            phantom: PhantomData,
            album_name,
            album_slug,
            band_name,
            band_slug,
            combined_lyrics,
            song,
            lyrics,
        })
    }
}
