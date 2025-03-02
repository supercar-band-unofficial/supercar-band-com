use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, PhotoAlbumWithPreviews };

#[derive(Template)]
#[template(path = "ui_modules/photo_album_list.html")]
pub struct PhotoAlbumListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    albums: Vec<PhotoAlbumWithPreviews>,
}
impl<'a> PhotoAlbumListTemplate<'a> {
    pub async fn new() -> Result<PhotoAlbumListTemplate<'a>, Box<dyn Error>> {

        let albums = database::get_all_photo_albums().await?;

        Ok(PhotoAlbumListTemplate {
            phantom: PhantomData,
            albums,
        })
    }
}

pub fn create_album_href(album: &PhotoAlbumWithPreviews) -> String {
    format!("/photos/{}/", album.slug)
}
