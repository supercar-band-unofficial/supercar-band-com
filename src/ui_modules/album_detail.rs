use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, Album, JoinedSongSlugs };

pub struct AlbumDetailParams {
    pub album: Album,
    pub band_slug: String,
}

#[derive(Template)]
#[template(path = "ui_modules/album_detail.html")]
pub struct AlbumDetailTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    pub album: Album,
    band_slug: String,
    songs: Vec<JoinedSongSlugs>,
}
impl<'a> AlbumDetailTemplate<'a> {
    pub async fn new(
        params: AlbumDetailParams,
    ) -> Result<AlbumDetailTemplate<'a>, Box<dyn Error>> {
        let AlbumDetailParams { album, band_slug } = params;

        let songs = database::get_song_slugs_by_ids(&album.song_ids()).await?;

        Ok(AlbumDetailTemplate {
            phantom: PhantomData,
            album,
            band_slug,
            songs,
        })
    }
}

pub fn create_song_href(band_slug: &str, album_slug: &str, song_slug: &str) -> String {
    format!("/lyrics/{}/{}/{}/", band_slug, album_slug, song_slug)
}
