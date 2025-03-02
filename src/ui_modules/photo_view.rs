use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, Photo };
use crate::util::format::make_content_links;

pub struct PhotoViewParams {
    pub photo_id: i32,
}

#[derive(Template)]
#[template(path = "ui_modules/photo_view.html")]
pub struct PhotoViewTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    pub photo: Photo,
}
impl<'a> PhotoViewTemplate<'a> {
    pub async fn new(
        params: PhotoViewParams,
    ) -> Result<PhotoViewTemplate<'a>, Box<dyn Error>> {
        let PhotoViewParams { photo_id } = params;

        let photo = database::get_photo_by_id(photo_id).await?;

        Ok(PhotoViewTemplate {
            phantom: PhantomData,
            photo,
        })
    }
}
