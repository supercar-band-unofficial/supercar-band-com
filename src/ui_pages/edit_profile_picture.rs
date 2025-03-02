use std::error::Error;
use askama::Template;
use garde::{ Report };
use uuid::Uuid;

use crate::database::{ self };
use crate::router::routes::edit_profile_picture::{ EditProfilePicturePageContext };
use crate::router::validation::report_has_field;
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::util::user::create_user_profile_href;

struct EditProfilePictureTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    temporary_profile_picture_filename: String,
    profile_picture_file_path: String,
    username: &'a str,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_profile_picture.html")]
pub struct EditProfilePicturePageTemplate<'a> {
    active_page: &'a str,
    content: EditProfilePictureTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditProfilePicturePageContext>,
}
impl<'a> EditProfilePicturePageTemplate<'a> {
    pub async fn new(
        context: &'a EditProfilePicturePageContext
    ) -> Result<EditProfilePicturePageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditProfilePicturePageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_profile_picture.html", block = "page_content")]
pub struct EditProfilePicturePageContentTemplate<'a> {
    content: EditProfilePictureTemplateCommon<'a>,
}
impl<'a> EditProfilePicturePageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditProfilePicturePageContext
    ) -> Result<EditProfilePicturePageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditProfilePicturePageContentTemplate {
            content,
        })
    }
}

fn get_cancel_href<'a>(content: &EditProfilePictureTemplateCommon<'a>) -> String {
    create_user_profile_href(content.username)
}

async fn create_common_params<'a>(context: &'a EditProfilePicturePageContext) -> Result<EditProfilePictureTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = context.user.is_some();
    let username: &'a str = if let Some(user) = &context.user {
        &user.username
    } else {
        ""
    };

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (
        temporary_profile_picture_filename, profile_picture_file_path,
    ) = if validation_alert.is_some() || !has_access {
        let profile_picture_file_path = if !context.params.temporary_profile_picture_filename.is_empty() {
            format!("/assets/images/tmp/{}", &context.params.temporary_profile_picture_filename)
        } else {
            String::from("")
        };
        (
            context.params.temporary_profile_picture_filename.clone(),
            profile_picture_file_path,
        )
    } else {
        let user = database::get_user_by_username(&username).await?;
        let cache_bust_id = Uuid::new_v4().to_string();
        (
            String::from(""),
            format!("/assets/images/profile-pictures/{}?cache-bust={}", user.profile_picture_filename, cache_bust_id),
        )
    };

    Ok(
        EditProfilePictureTemplateCommon {
            has_access,
            validation_alert,
            temporary_profile_picture_filename,
            profile_picture_file_path,
            username,
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
            if report_has_field(report, "temporary_profile_picture_filename") {
                message_html.push_str("<p>Please upload a jpeg or png file that is less than 6 megabytes large.</p>");
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
