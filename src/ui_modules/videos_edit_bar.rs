use std::error::Error;
use std::marker::PhantomData;
use askama::Template;

use crate::database::{ UserPermission };
use crate::router::routes::videos::{ VideosPageContext };

pub struct VideosEditBarParams<'a, VideosPageContext> {
    pub context: &'a VideosPageContext,
    pub video_category_contributor: Option<String>,
    pub video_contributor: Option<String>,
    pub video_title: String,
    pub video_category_title: String,
}

#[derive(Template)]
#[template(path = "ui_modules/videos_edit_bar.html")]
pub struct VideosEditBarTemplate<'a, VideosPageContext> {
    phantom: PhantomData<&'a VideosPageContext>,
    can_create_video_category: bool,
    can_edit_video_category: bool,
    can_delete_video_category: bool,
    can_create_video: bool,
    can_edit_video: bool,
    can_delete_video: bool,
    video_category_slug: &'a str,
    video_slug: &'a str,
    video_title: String,
    video_category_title: String,
}
impl<'a> VideosEditBarTemplate<'a, VideosPageContext> {
    pub async fn new(
        params: VideosEditBarParams<'a, VideosPageContext>,
    ) -> Result<VideosEditBarTemplate<'a, VideosPageContext>, Box<dyn Error>> {
        let VideosEditBarParams {
            context, video_category_contributor, video_contributor, video_title, video_category_title
        } = params;

        let mut can_create_video_category = false;
        let mut can_edit_video_category = false;
        let mut can_delete_video_category = false;
        let mut can_create_video = false;
        let mut can_edit_video = false;
        let mut can_delete_video = false;

        if let Some(user) = &context.user {
            can_create_video_category = user.permissions.contains(&UserPermission::CreateOwnVideoCategory);
            can_create_video = user.permissions.contains(&UserPermission::UploadOwnVideo);
            if !context.params.category.is_empty() {
                if let Some(video_category_contributor) = &video_category_contributor {
                    can_edit_video_category =
                        user.permissions.contains(&UserPermission::EditVideoCategory) || (
                            user.permissions.contains(&UserPermission::EditOwnVideoCategory) &&
                            video_category_contributor == &user.username
                        );
                    can_delete_video_category =
                        user.permissions.contains(&UserPermission::DeleteVideoCategory) || (
                            user.permissions.contains(&UserPermission::DeleteOwnVideoCategory) &&
                            video_category_contributor == &user.username
                        );
                }

                if !context.params.video.is_empty() {
                    if let Some(video_contributor) = &video_contributor {
                        can_edit_video =
                            user.permissions.contains(&UserPermission::EditVideo) || (
                                user.permissions.contains(&UserPermission::EditOwnVideo) &&
                                video_contributor == &user.username
                            );
                        can_delete_video =
                            user.permissions.contains(&UserPermission::DeleteVideo) || (
                                user.permissions.contains(&UserPermission::DeleteOwnVideo) &&
                                video_contributor == &user.username
                            );
                    }
                }
            }
        }

        Ok(VideosEditBarTemplate {
            phantom: PhantomData,
            can_create_video_category,
            can_edit_video_category,
            can_delete_video_category,
            can_create_video,
            can_edit_video,
            can_delete_video,
            video_category_slug: context.params.category.as_str(),
            video_slug: context.params.video.as_str(),
            video_title,
            video_category_title,
        })
    }
}
