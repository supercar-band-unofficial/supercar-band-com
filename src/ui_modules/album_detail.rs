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
    has_lyrics_booklet: bool,
}
impl<'a> AlbumDetailTemplate<'a> {
    pub async fn new(
        params: AlbumDetailParams,
    ) -> Result<AlbumDetailTemplate<'a>, Box<dyn Error>> {
        let AlbumDetailParams { album, band_slug } = params;

        let songs = database::get_song_slugs_by_ids(&album.song_ids()).await?;
        let images = database::get_lyrics_booklet_images(&band_slug, &album.album_slug).await;

        Ok(AlbumDetailTemplate {
            phantom: PhantomData,
            album,
            band_slug,
            songs,
            has_lyrics_booklet: images.len() > 0,
        })
    }
}

pub fn create_song_href(band_slug: &str, album_slug: &str, song_slug: &str) -> String {
    format!("/lyrics/{}/{}/{}/", band_slug, album_slug, song_slug)
}
