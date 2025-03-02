use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, JoinedSongTab };
use crate::util::format;
use crate::util::user::create_user_profile_href;

pub struct TabsSongDetailParams {
    pub band_id: i32,
    pub band_slug: String,
    pub band_name: String,
    pub song_slug: String,
    pub is_signed_in: bool,
}

#[derive(Template)]
#[template(path = "ui_modules/tabs_song_detail.html")]
pub struct TabsSongDetailTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    pub band_slug: String,
    pub band_name: String,
    pub song_slug: String,
    pub song_name: String,
    pub tabs: Vec<JoinedSongTab>,
    pub is_signed_in: bool,
}
impl<'a> TabsSongDetailTemplate<'a> {
    pub async fn new(
        params: TabsSongDetailParams,
    ) -> Result<TabsSongDetailTemplate<'a>, Box<dyn Error>> {
        let TabsSongDetailParams { band_id, band_slug, band_name, song_slug, is_signed_in } = params;

        let song = database::get_song_by_slug_and_band_id(&song_slug, band_id).await?;
        let tabs = database::get_song_tabs_by_song_id(song.id).await?;
        
        Ok(TabsSongDetailTemplate {
            phantom: PhantomData,
            band_slug,
            band_name,
            song_slug,
            song_name: song.song_name,
            tabs,
            is_signed_in,
        })
    }
}

fn create_tab_href(band_slug: &str, tab: &JoinedSongTab) -> String {
    format!("/tabs/{}/{}/{}/{}/", band_slug, tab.song_slug, format::to_kebab_case(tab.tab_type.to_string().as_str()), tab.username)
}
