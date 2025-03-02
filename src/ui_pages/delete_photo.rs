use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_photo::{ DeletePhotoPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeletePhotoTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    photo_album_slug: &'a str,
    photo_id: i32,
    photo_title: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_photo.html")]
pub struct DeletePhotoPageTemplate<'a> {
    active_page: &'a str,
    content: DeletePhotoTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeletePhotoPageContext>,
}
impl<'a> DeletePhotoPageTemplate<'a> {
    pub async fn new(
        context: &'a DeletePhotoPageContext
    ) -> Result<DeletePhotoPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeletePhotoPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_photo.html", block = "page_content")]
pub struct DeletePhotoPageContentTemplate<'a> {
    content: DeletePhotoTemplateCommon<'a>,
}
impl<'a> DeletePhotoPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeletePhotoPageContext
    ) -> Result<DeletePhotoPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeletePhotoPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeletePhotoTemplateCommon<'a>) -> String {
    return format!("/editor/delete/photo/{}/{}/",
        content.photo_album_slug, content.photo_id,
    );
}

async fn create_common_params<'a>(context: &'a DeletePhotoPageContext) -> Result<DeletePhotoTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = true;

    let photo_album_slug = &context.params.album;
    let photo_id = context.params.photo;
    let _ = database::get_photo_album_by_slug(photo_album_slug).await;
    let photo = database::get_photo_by_id(context.params.photo).await;

    let photo_missing_report = Some(
        create_simple_report(String::from("photo_missing"), String::from("Photo missing."))
    );
    let mut photo_title: String = String::from("");
    let validation_alert = get_validation_alert(
        if let Ok(photo) = photo {
            photo_title = photo.title;
            &context.params.validation_report
        } else {
            &photo_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeletePhotoTemplateCommon {
            has_access,
            validation_alert,
            photo_album_slug,
            photo_id,
            photo_title,
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
            if report_has_field(report, "photo_missing") {
                message_html.push_str("<p>The specified photo does not exist.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
