use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, VideoCategoryWithPreview };
use crate::util::video::get_video_thumbnail_url;

#[derive(Template)]
#[template(path = "ui_modules/video_category_list.html")]
pub struct VideoCategoryListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    categories: Vec<VideoCategoryWithPreview>,
}
impl<'a> VideoCategoryListTemplate<'a> {
    pub async fn new() -> Result<VideoCategoryListTemplate<'a>, Box<dyn Error>> {

        let categories = database::get_all_video_categories().await?;

        Ok(VideoCategoryListTemplate {
            phantom: PhantomData,
            categories,
        })
    }
}

pub fn create_category_href(category: &VideoCategoryWithPreview) -> String {
    format!("/videos/{}/", category.slug)
}
