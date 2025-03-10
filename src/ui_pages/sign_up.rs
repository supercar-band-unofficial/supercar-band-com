use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::ui_primitives::alert::AlertTemplate;
use crate::ui_primitives::captcha::CaptchaTemplate;
use crate::router::routes::sign_up::{ SignUpPageContext, SignUpPageParams };
use crate::router::validation::report_has_field;
use crate::util::captcha::{ generate_captcha, get_captcha_pow_challenge_by_id };

struct SignUpPageTemplateCommon<'a> {
    alert: Option<AlertTemplate<'a>>,
    no_script_warning: AlertTemplate<'a>,
    username: &'a str,
    first_name: &'a str,
    last_name: &'a str,
    captcha: CaptchaTemplate<'a>,
}

#[derive(Template)]
#[template(path = "ui_pages/sign_up.html")]
pub struct SignUpTemplate<'a> {
    active_page: &'a str,
    content: SignUpPageTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, SignUpPageContext>,
}
impl<'a> SignUpTemplate<'a> {
    pub async fn new(context: &'a SignUpPageContext) -> Result<SignUpTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        let content = create_common_params(context).await?;

        Ok(SignUpTemplate { active_page, content, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/sign_up.html", block = "page_content")]
pub struct SignUpContentTemplate<'a> {
    content: SignUpPageTemplateCommon<'a>,
}
impl<'a> SignUpContentTemplate<'a> {
    pub async fn new(context: &'a SignUpPageContext) -> Result<SignUpContentTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;

        Ok(SignUpContentTemplate { content })
    }
}

async fn create_common_params<'a>(context: &'a SignUpPageContext) -> Result<SignUpPageTemplateCommon<'a>, Box<dyn Error>> {
    let SignUpPageContext { params, .. } = context;
    let SignUpPageParams { username, first_name, last_name, validation_report, .. } = params;

    let captcha_id = generate_captcha()?;
    let pow_challenge = get_captcha_pow_challenge_by_id(&captcha_id).unwrap_or_else(|| String::from(""));

    let captcha = CaptchaTemplate {
        captcha_id,
        pow_challenge,
        form_id_prefix: "sign-up",
    };

    let no_script_warning = AlertTemplate {
        variant: "danger",
        message_html: String::from("<p>Unfortunately due to a large influx of bot registrations, this page requires Javascript in order to work properly. Unless you enable Javascript in your web browser, you will not be able to create an account.</p>"),
    };

    let alert = get_validation_alert(validation_report);

    Ok(
        SignUpPageTemplateCommon {
            username,
            first_name,
            last_name,
            alert,
            no_script_warning,
            captcha,
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
            if report_has_field(report, "first_name") {
                message_html.push_str("<p><strong>First Name:</strong> Must be 1 to 30 characters long.</p>");
            }
            if report_has_field(report, "last_name") {
                message_html.push_str("<p><strong>Last Name:</strong> Cannot be more than 30 characters long.</p>");
            }
            if report_has_field(report, "username") {
                message_html.push_str("<p><strong>Username:</strong> Must be 1 to 30 characters long, can include letters and numbers.</p>");
            }
            if report_has_field(report, "username_taken") {
                message_html.push_str("<p><strong>Username:</strong> The username you have entered is already taken. Please choose a different one.</p>");
            }
            if report_has_field(report, "password") {
                message_html.push_str("<p><strong>Password:</strong> Must be at least 1 character long, cannot include invisible characters.</p>");
            }
            if report_has_field(report, "captcha_entry") {
                message_html.push_str("<p><strong>Human Verification:</strong> The correct letters were not entered.</p>");
            }
            if report_has_field(report, "rate_limit") {
                message_html.push_str("<p>A recent request to create an account has been made from your computer (or immediate network). Please try again tomorrow.</p>");
            }
            

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
