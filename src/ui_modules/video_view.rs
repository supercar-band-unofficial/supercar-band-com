use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, Video };
use crate::util::format::make_content_links;
use crate::util::video::create_video_embed_iframe;

pub struct VideoViewParams<'a> {
    pub video_slug: &'a str,
    pub category_id: i32,
}

#[derive(Template)]
#[template(path = "ui_modules/video_view.html")]
pub struct VideoViewTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    pub video: Video,
}
impl<'a> VideoViewTemplate<'a> {
    pub async fn new(
        params: VideoViewParams<'a>,
    ) -> Result<VideoViewTemplate<'a>, Box<dyn Error>> {
        let VideoViewParams { video_slug, category_id } = params;

        let video = database::get_video_by_slug_and_category_id(video_slug, category_id).await?;

        Ok(VideoViewTemplate {
            phantom: PhantomData,
            video,
        })
    }
}
