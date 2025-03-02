use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::util::user::create_user_profile_href;
use crate::router::routes::edit_profile_info::{ EditProfileInfoPageContext };
use crate::router::validation::report_has_field;

struct EditProfileInfoTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    username: &'a str,
    first_name: String,
    last_name: String,
    email: String,
    gender: String,
    country: String,
    about_me: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_profile_info.html")]
pub struct EditProfileInfoPageTemplate<'a> {
    active_page: &'a str,
    content: EditProfileInfoTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditProfileInfoPageContext>,
}
impl<'a> EditProfileInfoPageTemplate<'a> {
    pub async fn new(
        context: &'a EditProfileInfoPageContext
    ) -> Result<EditProfileInfoPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditProfileInfoPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_profile_info.html", block = "page_content")]
pub struct EditProfileInfoPageContentTemplate<'a> {
    content: EditProfileInfoTemplateCommon<'a>,
}
impl<'a> EditProfileInfoPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditProfileInfoPageContext
    ) -> Result<EditProfileInfoPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditProfileInfoPageContentTemplate {
            content,
        })
    }
}

fn get_cancel_href<'a>(content: &EditProfileInfoTemplateCommon<'a>) -> String {
    create_user_profile_href(content.username)
}

async fn create_common_params<'a>(context: &'a EditProfileInfoPageContext) -> Result<EditProfileInfoTemplateCommon<'a>, Box<dyn Error>> {

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

    let (first_name, last_name, email, gender, country, about_me) = if validation_alert.is_some() || !has_access {
        (
            context.params.first_name.clone(),
            context.params.last_name.clone(),
            context.params.email.clone(),
            context.params.gender.clone(),
            context.params.country.clone(),
            context.params.about_me.clone(),
        )
    } else {
        let user = database::get_user_by_username(&username).await?;
        (
            user.first_name,
            user.last_name,
            user.email,
            user.gender.to_string(),
            user.country,
            user.about_me,
        )
    };

    Ok(
        EditProfileInfoTemplateCommon {
            has_access,
            validation_alert,
            username,
            first_name,
            last_name,
            email,
            gender,
            country,
            about_me,
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
            if report_has_field(report, "email") {
                message_html.push_str("<p><strong>Email:</strong> Please enter a valid email address.</p>");
            }
            if report_has_field(report, "first_name") {
                message_html.push_str("<p><strong>First Name:</strong> This field is required.</p>");
            }
            if report_has_field(report, "last_name") {
                message_html.push_str("<p><strong>Last Name:</strong> Invalid input.</p>");
            }
            if report_has_field(report, "gender") {
                message_html.push_str("<p><strong>Gender:</strong> Invalid input.</p>");
            }
            if report_has_field(report, "country") {
                message_html.push_str("<p><strong>Country:</strong> Invalid input.</p>");
            }
            if report_has_field(report, "about_me") {
                message_html.push_str("<p><strong>About Me:</strong> Invalid input.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
