use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, AlbumSearchResult, Band, SongSearchResult };

pub struct LyricsSearchResultsParams {
    pub search: String,
}

#[derive(Template)]
#[template(path = "ui_modules/lyrics_search_results.html")]
pub struct LyricsSearchResultsTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    search: String,
    songs: Vec<SongSearchResult>,
    albums: Vec<AlbumSearchResult>,
    bands: Vec<Band>,
}
impl<'a> LyricsSearchResultsTemplate<'a> {
    pub async fn new(
        params: LyricsSearchResultsParams,
    ) -> Result<LyricsSearchResultsTemplate<'a>, Box<dyn Error>> {
        let LyricsSearchResultsParams { search } = params;

        let songs = database::find_songs_with_translations_by_name(&search).await?;
        let albums = database::find_albums_by_name(&search).await?;
        let bands = database::find_bands_by_name(&search).await?;

        Ok(LyricsSearchResultsTemplate {
            phantom: PhantomData,
            search,
            songs,
            albums,
            bands,
        })
    }
}

pub fn create_song_href(search_result: &SongSearchResult) -> String {
    format!("/lyrics/{}/{}/{}/", search_result.band_slug, search_result.album_slug, search_result.song_slug)
}

pub fn create_album_href(search_result: &AlbumSearchResult) -> String {
    format!("/lyrics/{}/{}/", search_result.band_slug, search_result.album_slug)
}
