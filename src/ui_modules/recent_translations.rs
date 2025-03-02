use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, RecentLyricTranslation };

pub struct RecentTranslationsParams {
    pub band_id: i32,
}

#[derive(Template)]
#[template(path = "ui_modules/recent_translations.html")]
pub struct RecentTranslationsTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    translations: Vec<RecentLyricTranslation>,
}
impl<'a> RecentTranslationsTemplate<'a> {
    pub async fn new(
        params: RecentTranslationsParams,
    ) -> Result<RecentTranslationsTemplate<'a>, Box<dyn Error>> {
        let RecentTranslationsParams { band_id } = params;

        let translations = database::get_recent_lyric_translations_by_band_id(band_id).await?;

        Ok(RecentTranslationsTemplate {
            phantom: PhantomData,
            translations,
        })
    }
}

fn create_song_href(song: &RecentLyricTranslation) -> String {
    format!("/lyrics/{}/{}/{}/", song.band_slug, song.album_slug, song.song_slug)
}
