use std::error::Error;
use askama::Template;

use crate::database::{ self, CommentSectionName };
use crate::ui_modules::comment_section::{ CommentSectionParams, CommentSectionTemplate };
use crate::ui_modules::photo_album_list::{ PhotoAlbumListTemplate };
use crate::ui_modules::photo_list::{ PhotoListTemplate, PhotoListParams };
use crate::ui_modules::photo_view::{ PhotoViewTemplate, PhotoViewParams };
use crate::ui_modules::photos_edit_bar::{ PhotosEditBarTemplate, PhotosEditBarParams };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::util::format::make_content_links;
use crate::router::routes::photos::{ PhotosPageContext };

struct PhotosTemplateCommon<'a> {
    seo_title: String,
    comment_section: Option<CommentSectionTemplate<'a, PhotosPageContext>>,
    photo_album_description: Option<String>,
    photo_album_list: Option<PhotoAlbumListTemplate<'a>>,
    photo_album_title: Option<String>,
    photo_album_slug: String,
    photos_edit_bar: PhotosEditBarTemplate<'a, PhotosPageContext>,
    photo_list: Option<PhotoListTemplate<'a>>,
    photo_view: Option<PhotoViewTemplate<'a>>,
    photo_title: String,
}

#[derive(Template)]
#[template(path = "ui_pages/photos.html")]
pub struct PhotosTemplate<'a> {
    active_page: &'a str,
    content: PhotosTemplateCommon<'a>,
    needs_title_update: bool,
    sidebar: SidebarTemplate<'a, PhotosPageContext>,
}
impl<'a> PhotosTemplate<'a> {
    pub async fn new(context: &'a PhotosPageContext) -> Result<PhotosTemplate<'a>, Box<dyn Error>> {
        let active_page = "photos";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let content = create_common_params(context).await?;
        Ok(PhotosTemplate {
            active_page, content, sidebar, needs_title_update: false,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/photos.html", block = "page_content")]
pub struct PhotosContentTemplate<'a> {
    content: PhotosTemplateCommon<'a>,
    needs_title_update: bool,
}
impl<'a> PhotosContentTemplate<'a> {
    pub async fn new(context: &'a PhotosPageContext) -> Result<PhotosContentTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(PhotosContentTemplate {
            content, needs_title_update: true,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/photos.html", block = "page_comments")]
pub struct PhotosCommentsTemplate<'a> {
    content: PhotosTemplateCommon<'a>,
}
impl<'a> PhotosCommentsTemplate<'a> {
    pub async fn new(context: &'a PhotosPageContext) -> Result<PhotosCommentsTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(PhotosCommentsTemplate {
            content,
        })
    }
}

async fn create_common_params<'a>(context: &'a PhotosPageContext) -> Result<PhotosTemplateCommon<'a>, Box<dyn Error>> {
    let seo_title: String;
    let mut comment_section = None;
    let mut photo_album_list = None;
    let mut photo_list = None;
    let mut photo_view = None;
    let mut photo_album_title = None;
    let mut photo_album_description = None;
    let mut photo_album_contributor = None;
    let mut photo_title = String::from("");

    if context.params.photo > -1 {
        let album = database::get_photo_album_by_slug(
            &context.params.album
        ).await?;

        photo_album_title = Some(album.title);
        photo_album_contributor = Some(album.username.clone());

        let photo_id = context.params.photo;

        photo_view = Some(
            PhotoViewTemplate::new(
                PhotoViewParams {
                    photo_id,
                }
            ).await?
        );

        let photo = &photo_view.as_ref().unwrap().photo;
        photo_title = if photo.title.len() > 0 { photo.title.clone() } else { String::from("Untitled Photo") };

        comment_section = Some(
            CommentSectionTemplate::<PhotosPageContext>::new(
                CommentSectionParams {
                    context,
                    section: &CommentSectionName::Photos,
                    section_tag_id: Some(photo_id),
                    page_number: context.params.comments_page,
                }
            ).await?
        );

        seo_title = format!("");
    } else if !context.params.album.is_empty() {
        let album = database::get_photo_album_by_slug(
            &context.params.album
        ).await?;

        photo_album_title = Some(album.title);
        photo_album_contributor = Some(album.username.clone());
        if !album.description.is_empty() {
            photo_album_description = Some(album.description);
        }

        photo_list = Some(
            PhotoListTemplate::new(
                PhotoListParams {
                    photo_album_slug: &context.params.album
                }
            ).await?
        );

        seo_title = format!(" for Photo Album {}", &photo_album_title.as_ref().unwrap());
    } else {
        photo_album_list = Some(
            PhotoAlbumListTemplate::new().await?
        );
        
        seo_title = format!("");
    }


    let photos_edit_bar = PhotosEditBarTemplate::new(
        PhotosEditBarParams {
            context,
            photo_album_contributor: if photo_album_contributor.is_some() { Some(photo_album_contributor.unwrap().clone()) } else { None },
            photo_contributor: if photo_view.is_some() { Some(photo_view.as_ref().unwrap().photo.username.clone()) } else { None },
            photo_album_title: if photo_album_title.is_some() { photo_album_title.as_ref().unwrap().clone() } else { String::from("") },
            photo_title: if photo_view.is_some() { photo_view.as_ref().unwrap().photo.title.clone() } else { String::from("") },
        }
    ).await?;

    Ok(
        PhotosTemplateCommon {
            seo_title,
            comment_section,
            photo_album_description,
            photo_album_list,
            photo_album_title,
            photo_album_slug: context.params.album.clone(),
            photos_edit_bar,
            photo_list,
            photo_view,
            photo_title,
        }
    )
}
