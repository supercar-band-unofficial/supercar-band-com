use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, AlbumSummary };

pub struct AlbumListParams {
    pub band_id: i32,
    pub band_slug: String,
}

#[derive(Template)]
#[template(path = "ui_modules/album_list.html")]
pub struct AlbumListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    albums: Vec<AlbumSummary>,
    band_slug: String,
}
impl<'a> AlbumListTemplate<'a> {
    pub async fn new(
        params: AlbumListParams,
    ) -> Result<AlbumListTemplate<'a>, Box<dyn Error>> {
        let AlbumListParams { band_id, band_slug } = params;

        let albums = database::get_album_summaries_by_band_id(band_id).await?;

        Ok(AlbumListTemplate {
            phantom: PhantomData,
            albums,
            band_slug,
        })
    }
}

pub fn create_album_href(band_slug: &str, album_slug: &str) -> String {
    format!("/lyrics/{}/{}/", band_slug, album_slug)
}
