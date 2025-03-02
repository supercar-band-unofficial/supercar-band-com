use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::edit_photo_album::{ EditPhotoAlbumPageContext };
use crate::router::validation::report_has_field;

struct EditPhotoAlbumTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    selected_album_slug: String,
    title: String,
    description: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_photo_album.html")]
pub struct EditPhotoAlbumPageTemplate<'a> {
    active_page: &'a str,
    content: EditPhotoAlbumTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditPhotoAlbumPageContext>,
}
impl<'a> EditPhotoAlbumPageTemplate<'a> {
    pub async fn new(
        context: &'a EditPhotoAlbumPageContext
    ) -> Result<EditPhotoAlbumPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditPhotoAlbumPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_photo_album.html", block = "page_content")]
pub struct EditPhotoAlbumPageContentTemplate<'a> {
    content: EditPhotoAlbumTemplateCommon<'a>,
}
impl<'a> EditPhotoAlbumPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditPhotoAlbumPageContext
    ) -> Result<EditPhotoAlbumPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditPhotoAlbumPageContentTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditPhotoAlbumTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Create Photo Album",
        _ => "Edit Photo Album",
    }
}

fn get_cancel_href<'a>(content: &EditPhotoAlbumTemplateCommon<'a>) -> String {
    if content.is_create {
        return format!("/photos/");
    }
    return format!("/photos/{}/",
        content.selected_album_slug,
    );
}

fn get_submit_action<'a>(content: &EditPhotoAlbumTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/photo-album/");
    }
    return format!("/editor/update/photo-album/{}/",
        content.selected_album_slug
    );
}

async fn create_common_params<'a>(context: &'a EditPhotoAlbumPageContext) -> Result<EditPhotoAlbumTemplateCommon<'a>, Box<dyn Error>> {

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_album_slug = &context.params.album;

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (title, description) = if is_create || validation_alert.is_some() {
        (
            context.params.title.clone(),
            context.params.description.clone(),
        )
    } else {
        let album = database::get_photo_album_by_slug(selected_album_slug).await?;
        (
            album.title,
            album.description,
        )
    };

    Ok(
        EditPhotoAlbumTemplateCommon {
            is_create,
            has_access,
            validation_alert,
            title,
            description,
            selected_album_slug: selected_album_slug.to_string(),
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
            if report_has_field(report, "photo_album_exist") {
                message_html.push_str("<p>A photo album with this name already exists. Please edit the existing photo album.</p>");
            }
            if report_has_field(report, "photo_album_missing") {
                message_html.push_str("<p>The photo album doesn't exist. It may have been deleted after visiting this page.</p>");
            }
            if report_has_field(report, "title") {
                message_html.push_str("<p><strong>Title:</strong> This field is required.</p>");
            }
            if report_has_field(report, "description") {
                message_html.push_str("<p><strong>Description:</strong> Invalid input.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
