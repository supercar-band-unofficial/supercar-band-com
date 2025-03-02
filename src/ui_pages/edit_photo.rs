use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self, PhotoAlbumWithPreviews };
use crate::router::routes::edit_photo::{ EditPhotoPageContext };
use crate::router::validation::report_has_field;
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;

struct EditPhotoTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    albums: Vec<PhotoAlbumWithPreviews>,
    selected_album_slug: String,
    selected_photo_id: i32,
    validation_alert: Option<AlertTemplate<'a>>,
    title: String,
    description: String,
    temporary_photo_filename: String,
    photo_file_path: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_photo.html")]
pub struct EditPhotoPageTemplate<'a> {
    active_page: &'a str,
    content: EditPhotoTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditPhotoPageContext>,
}
impl<'a> EditPhotoPageTemplate<'a> {
    pub async fn new(
        context: &'a EditPhotoPageContext
    ) -> Result<EditPhotoPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditPhotoPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_photo.html", block = "page_content")]
pub struct EditPhotoPageContentTemplate<'a> {
    content: EditPhotoTemplateCommon<'a>,
}
impl<'a> EditPhotoPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditPhotoPageContext
    ) -> Result<EditPhotoPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditPhotoPageContentTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditPhotoTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Upload Photo",
        _ => "Edit Photo",
    }
}

fn get_submit_action<'a>(content: &EditPhotoTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/photo/");
    }
    return format!("/editor/update/photo/{}/{}/",
        content.selected_album_slug, content.selected_photo_id,
    );
}

fn get_cancel_href<'a>(content: &EditPhotoTemplateCommon<'a>) -> String {
    if content.is_create {
        return format!("/photos/{}/", content.selected_album_slug);
    }
    return format!("/photos/{}/{}/",
        content.selected_album_slug, content.selected_photo_id,
    );
}

async fn create_common_params<'a>(context: &'a EditPhotoPageContext) -> Result<EditPhotoTemplateCommon<'a>, Box<dyn Error>> {
    let albums = database::get_all_photo_albums().await?;

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_album_slug = &context.params.album;
    let selected_photo_id = context.params.photo;

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (
        title, description, temporary_photo_filename, photo_file_path,
    ) = if is_create || validation_alert.is_some() {
        let photo_file_path = if !context.params.temporary_photo_filename.is_empty() {
            format!("/assets/images/tmp/{}", &context.params.temporary_photo_filename)
        } else {
            String::from("")
        };
        (
            context.params.title.clone(),
            context.params.description.clone(),
            context.params.temporary_photo_filename.clone(),
            photo_file_path,
        )
    } else {
        let _ = database::get_photo_album_by_slug(selected_album_slug).await?;
        let photo = database::get_photo_by_id(selected_photo_id).await?;
        (
            photo.title,
            photo.description,
            String::from(""),
            format!("/assets/images/photos/{}", photo.photo_filename),
        )
    };

    Ok(
        EditPhotoTemplateCommon {
            is_create,
            has_access,

            albums,
            selected_album_slug: selected_album_slug.to_string(),
            selected_photo_id,
            validation_alert,
            title,
            description,
            temporary_photo_filename,
            photo_file_path,
        }
    )
}

fn get_validation_alert<'a>(report: &Option<Report>) -> Option<AlertTemplate<'a>> {
    match report {
        Some(report) => {
            let mut message_html: String = "".to_owned();

            if report_has_field(report, "server_error") {
                message_html.push_str("<p>A system error occurred. Please notify the site admins if this continues to happen.</p>");
            }
            if report_has_field(report, "forbidden") {
                message_html.push_str("<p>You do not have sufficient permissions to use this form.</p>");
            }
            if report_has_field(report, "photo_exist") {
                message_html.push_str("<p>Another photo with the same name already exists.</p>");
            }
            if report_has_field(report, "album_missing") {
                message_html.push_str("<p>The photo album you are adding to does not exist. Maybe it was deleted since loading this page.</p>");
            }
            if report_has_field(report, "title") {
                message_html.push_str("<p><strong>Title:</strong> Invalid entry.</p>");
            }
            if report_has_field(report, "description") {
                message_html.push_str("<p><strong>Description:</strong> Invalid entry.</p>");
            }
            if report_has_field(report, "temporary_photo_filename") {
                message_html.push_str("<p><strong>Photo:</strong> Please upload a jpeg or png file that is less than 12 megabytes large.</p>");
            }
            if report_has_field(report, "image_transfer") {
                message_html.push_str("<p>Image upload failed. Please notify the site admins if this continues to happen.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
