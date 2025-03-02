use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_photo_album::{ DeletePhotoAlbumPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeletePhotoAlbumTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    photo_album_slug: &'a str,
    photo_album_title: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_photo_album.html")]
pub struct DeletePhotoAlbumPageTemplate<'a> {
    active_page: &'a str,
    content: DeletePhotoAlbumTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeletePhotoAlbumPageContext>,
}
impl<'a> DeletePhotoAlbumPageTemplate<'a> {
    pub async fn new(
        context: &'a DeletePhotoAlbumPageContext
    ) -> Result<DeletePhotoAlbumPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeletePhotoAlbumPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_photo_album.html", block = "page_content")]
pub struct DeletePhotoAlbumPageContentTemplate<'a> {
    content: DeletePhotoAlbumTemplateCommon<'a>,
}
impl<'a> DeletePhotoAlbumPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeletePhotoAlbumPageContext
    ) -> Result<DeletePhotoAlbumPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeletePhotoAlbumPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeletePhotoAlbumTemplateCommon<'a>) -> String {
    return format!("/editor/delete/photo-album/{}/",
        content.photo_album_slug,
    );
}

async fn create_common_params<'a>(context: &'a DeletePhotoAlbumPageContext) -> Result<DeletePhotoAlbumTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = true;

    let photo_album_slug = &context.params.album;
    let photo_album = database::get_photo_album_by_slug(photo_album_slug).await;

    let photo_album_missing_report = Some(
        create_simple_report(String::from("photo_album_missing"), String::from("Album missing."))
    );
    let mut photo_album_title: String = String::from("");
    let validation_alert = get_validation_alert(
        if let Ok(photo_album) = photo_album {
            photo_album_title = photo_album.title;
            &context.params.validation_report
        } else {
            &photo_album_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeletePhotoAlbumTemplateCommon {
            has_access,
            validation_alert,
            photo_album_slug,
            photo_album_title,
        }
    )
}

fn get_validation_alert<'a>(report: &Option<Report>) -> Option<AlertTemplate<'a>> {
    match report {
        Some(report) => {
            let mut message_html: String = "".to_owned();

            if report_has_field(report, "server_error") {
                message_html.push_str("<p>A system error occurred. Please try again later.</p>");
            }
            if report_has_field(report, "forbidden") {
                message_html.push_str("<p>You do not have sufficient permissions to use this form.</p>");
            }
            if report_has_field(report, "photo_album_missing") {
                message_html.push_str("<p>The specified photo album does not exist.</p>");
            }
            if report_has_field(report, "photos_exist") {
                message_html.push_str("<p>The photo album has photos inside of it, and cannot be deleted until all photos inside of it are deleted.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
