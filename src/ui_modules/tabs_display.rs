use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, SongTab, SongTabType };
use crate::util::format;
use crate::util::user::create_user_profile_href;

pub struct TabsDisplayParams {
    pub band_id: i32,
    pub band_slug: String,
    pub band_name: String,
    pub song_slug: String,
    pub tab_type: String,
    pub contributor: String,
}

#[derive(Template)]
#[template(path = "ui_modules/tabs_display.html")]
pub struct TabsDisplayTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    pub band_slug: String,
    pub band_name: String,
    pub song_slug: String,
    pub song_name: String,
    pub contributor: String,
    pub tab: SongTab,
}
impl<'a> TabsDisplayTemplate<'a> {
    pub async fn new(
        params: TabsDisplayParams,
    ) -> Result<TabsDisplayTemplate<'a>, Box<dyn Error>> {
        let TabsDisplayParams { band_id, band_slug, band_name, song_slug, tab_type, contributor } = params;

        let song = database::get_song_by_slug_and_band_id(&song_slug, band_id).await?;
        let tab = database::get_song_tab_by_username_type_and_song_id(
            &contributor,
            &format::to_snake_case(&tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown),
            song.id
        ).await?;
        
        Ok(TabsDisplayTemplate {
            phantom: PhantomData,
            band_slug,
            band_name,
            song_slug,
            song_name: song.song_name,
            contributor,
            tab,
        })
    }
}
