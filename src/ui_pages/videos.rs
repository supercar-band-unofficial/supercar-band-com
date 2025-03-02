use std::error::Error;
use askama::Template;

use crate::database::{ self, CommentSectionName };
use crate::ui_modules::comment_section::{ CommentSectionParams, CommentSectionTemplate };
use crate::ui_modules::video_category_list::{ VideoCategoryListTemplate };
use crate::ui_modules::video_list::{ VideoListTemplate, VideoListParams };
use crate::ui_modules::video_view::{ VideoViewTemplate, VideoViewParams };
use crate::ui_modules::videos_edit_bar::{ VideosEditBarTemplate, VideosEditBarParams };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::util::format::make_content_links;
use crate::router::routes::videos::{ VideosPageContext };

struct VideosTemplateCommon<'a> {
    seo_title: String,
    comment_section: Option<CommentSectionTemplate<'a, VideosPageContext>>,
    video_category_description: Option<String>,
    video_category_list: Option<VideoCategoryListTemplate<'a>>,
    video_category_title: Option<String>,
    video_category_slug: String,
    videos_edit_bar: VideosEditBarTemplate<'a, VideosPageContext>,
    video_list: Option<VideoListTemplate<'a>>,
    video_view: Option<VideoViewTemplate<'a>>,
    video_title: String,
}

#[derive(Template)]
#[template(path = "ui_pages/videos.html")]
pub struct VideosTemplate<'a> {
    active_page: &'a str,
    content: VideosTemplateCommon<'a>,
    needs_title_update: bool,
    sidebar: SidebarTemplate<'a, VideosPageContext>,
}
impl<'a> VideosTemplate<'a> {
    pub async fn new(context: &'a VideosPageContext) -> Result<VideosTemplate<'a>, Box<dyn Error>> {
        let active_page = "videos";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let content = create_common_params(context).await?;
        Ok(VideosTemplate {
            active_page, content, sidebar, needs_title_update: false,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/videos.html", block = "page_content")]
pub struct VideosContentTemplate<'a> {
    content: VideosTemplateCommon<'a>,
    needs_title_update: bool,
}
impl<'a> VideosContentTemplate<'a> {
    pub async fn new(context: &'a VideosPageContext) -> Result<VideosContentTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(VideosContentTemplate {
            content, needs_title_update: true,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/videos.html", block = "page_comments")]
pub struct VideosCommentsTemplate<'a> {
    content: VideosTemplateCommon<'a>,
}
impl<'a> VideosCommentsTemplate<'a> {
    pub async fn new(context: &'a VideosPageContext) -> Result<VideosCommentsTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(VideosCommentsTemplate {
            content,
        })
    }
}

async fn create_common_params<'a>(context: &'a VideosPageContext) -> Result<VideosTemplateCommon<'a>, Box<dyn Error>> {
    let seo_title: String;
    let mut comment_section = None;
    let mut video_category_list = None;
    let mut video_list = None;
    let mut video_view = None;
    let mut video_category_title = None;
    let mut video_category_description = None;
    let mut video_category_contributor = None;
    let mut video_title = String::from("");

    if !context.params.video.is_empty() {
        let category = database::get_video_category_by_slug(
            &context.params.category
        ).await?;

        video_category_title = Some(category.title);
        video_category_contributor = Some(category.username.clone());

        let video_slug = &context.params.video;

        video_view = Some(
            VideoViewTemplate::new(
                VideoViewParams {
                    video_slug,
                    category_id: category.id,
                }
            ).await?
        );

        let video = &video_view.as_ref().unwrap().video;
        video_title = if video.title.len() > 0 { video.title.clone() } else { String::from("Untitled Video") };

        comment_section = Some(
            CommentSectionTemplate::<VideosPageContext>::new(
                CommentSectionParams {
                    context,
                    section: &CommentSectionName::Videos,
                    section_tag_id: Some(video.id),
                    page_number: context.params.comments_page,
                }
            ).await?
        );

        seo_title = format!(" for video {}", context.params.video);
    } else if !context.params.category.is_empty() {
        let category = database::get_video_category_by_slug(
            &context.params.category
        ).await?;

        video_category_title = Some(category.title);
        video_category_contributor = Some(category.username.clone());
        if !category.description.is_empty() {
            video_category_description = Some(category.description);
        }

        video_list = Some(
            VideoListTemplate::new(
                VideoListParams {
                    video_category_slug: &context.params.category
                }
            ).await?
        );

        seo_title = format!(" for video category {}", video_category_title.as_ref().unwrap());
    } else {
        video_category_list = Some(
            VideoCategoryListTemplate::new().await?
        );
        
        seo_title = format!("");
    }

    let videos_edit_bar = VideosEditBarTemplate::new(
        VideosEditBarParams {
            context,
            video_category_contributor: if video_category_contributor.is_some() { Some(video_category_contributor.unwrap().clone()) } else { None },
            video_contributor: if video_view.is_some() { Some(video_view.as_ref().unwrap().video.username.clone()) } else { None },
            video_category_title: if video_category_title.is_some() { video_category_title.as_ref().unwrap().clone() } else { String::from("") },
            video_title: if video_view.is_some() { video_view.as_ref().unwrap().video.title.clone() } else { String::from("") },
        }
    ).await?;

    Ok(
        VideosTemplateCommon {
            seo_title,
            comment_section,
            video_category_description,
            video_category_list,
            video_category_title,
            video_category_slug: context.params.category.clone(),
            videos_edit_bar,
            video_list,
            video_view,
            video_title,
        }
    )
}
