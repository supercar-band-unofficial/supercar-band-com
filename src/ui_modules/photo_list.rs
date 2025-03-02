use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, Photo };

pub struct PhotoListParams<'a> {
    pub photo_album_slug: &'a str,
}

#[derive(Template)]
#[template(path = "ui_modules/photo_list.html")]
pub struct PhotoListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    photo_album_slug: &'a str,
    photos: Vec<Photo>,
}
impl<'a> PhotoListTemplate<'a> {
    pub async fn new(
        params: PhotoListParams<'a>,
    ) -> Result<PhotoListTemplate<'a>, Box<dyn Error>> {
        let PhotoListParams { photo_album_slug } = params;

        let photos = database::get_photos_by_photo_album_slug(photo_album_slug).await?;

        Ok(PhotoListTemplate {
            phantom: PhantomData,
            photo_album_slug,
            photos,
        })
    }
}

pub fn create_photo_href<'a>(photo_album_slug: &'a str, photo: &Photo) -> String {
    format!("/photos/{}/{}/", photo_album_slug, photo.id)
}
