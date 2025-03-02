use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::util::user::create_user_profile_href;
use crate::router::routes::edit_profile_password::{ EditProfilePasswordPageContext };
use crate::router::validation::report_has_field;

struct EditProfilePasswordTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    username: &'a str,
    new_password: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_profile_password.html")]
pub struct EditProfilePasswordPageTemplate<'a> {
    active_page: &'a str,
    content: EditProfilePasswordTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditProfilePasswordPageContext>,
}
impl<'a> EditProfilePasswordPageTemplate<'a> {
    pub async fn new(
        context: &'a EditProfilePasswordPageContext
    ) -> Result<EditProfilePasswordPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditProfilePasswordPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_profile_password.html", block = "page_content")]
pub struct EditProfilePasswordPageContentTemplate<'a> {
    content: EditProfilePasswordTemplateCommon<'a>,
}
impl<'a> EditProfilePasswordPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditProfilePasswordPageContext
    ) -> Result<EditProfilePasswordPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditProfilePasswordPageContentTemplate {
            content,
        })
    }
}

fn get_cancel_href<'a>(content: &EditProfilePasswordTemplateCommon<'a>) -> String {
    create_user_profile_href(content.username)
}

async fn create_common_params<'a>(context: &'a EditProfilePasswordPageContext) -> Result<EditProfilePasswordTemplateCommon<'a>, Box<dyn Error>> {

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

    let new_password = if validation_alert.is_some() || !has_access {
        context.params.new_password.clone()
    } else {
        String::from("")
    };

    Ok(
        EditProfilePasswordTemplateCommon {
            has_access,
            validation_alert,
            username,
            new_password,
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
            if report_has_field(report, "current_password") {
                message_html.push_str("<p><strong>Current Password:</strong> Your current password was not entered correctly.</p>");
            }
            if report_has_field(report, "new_password") {
                message_html.push_str("<p><strong>New Password:</strong> Must be at least 1 character long, cannot include invisible characters.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
