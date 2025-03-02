use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ self, Video };
use crate::util::video::get_video_thumbnail_url;

pub struct VideoListParams<'a> {
    pub video_category_slug: &'a str,
}

#[derive(Template)]
#[template(path = "ui_modules/video_list.html")]
pub struct VideoListTemplate<'a> {
    phantom: PhantomData<&'a ()>,
    video_category_slug: &'a str,
    videos: Vec<Video>,
}
impl<'a> VideoListTemplate<'a> {
    pub async fn new(
        params: VideoListParams<'a>,
    ) -> Result<VideoListTemplate<'a>, Box<dyn Error>> {
        let VideoListParams { video_category_slug } = params;

        let videos = database::get_videos_by_video_category_slug(video_category_slug).await?;

        Ok(VideoListTemplate {
            phantom: PhantomData,
            video_category_slug,
            videos,
        })
    }
}

pub fn create_video_href<'a>(video_category_slug: &'a str, video: &Video) -> String {
    format!("/videos/{}/{}/", video_category_slug, video.slug)
}
