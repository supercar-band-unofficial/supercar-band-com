use std::error::Error;
use std::io;
use askama::Template;
use garde::{ Report };

use crate::router::routes::forgot_password::ForgotPasswordPageContext;
use crate::router::validation::{ report_has_field };
use crate::ui_modules::sidebar::{ SidebarParams, SidebarTemplate };
use crate::ui_primitives::alert::{ AlertTemplate };
use crate::ui_primitives::captcha::{ CaptchaTemplate };
use crate::util::captcha::{ generate_captcha };
use crate::util::password_reset_session::get_password_reset_session_username;

struct ForgotPasswordTemplateCommon<'a> {
    validation_alert: Option<AlertTemplate<'a>>,
    captcha: Option<CaptchaTemplate<'a>>,
    username: &'a str,
    session: &'a str,
    is_reset_success: bool,
}

#[derive(Template)]
#[template(path = "ui_pages/forgot_password.html")]
pub struct ForgotPasswordPageTemplate<'a> {
    active_page: &'a str,
    content: ForgotPasswordTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, ForgotPasswordPageContext>,
}
impl<'a> ForgotPasswordPageTemplate<'a> {
    pub async fn new(context: &'a ForgotPasswordPageContext) -> Result<ForgotPasswordPageTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";
        let content = create_common_params(context).await?;
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;
        Ok(ForgotPasswordPageTemplate { active_page, content, sidebar })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/forgot_password.html", block = "page_content")]
pub struct ForgotPasswordPageContentTemplate<'a> {
    content: ForgotPasswordTemplateCommon<'a>,
}
impl<'a> ForgotPasswordPageContentTemplate<'a> {
    pub async fn new(context: &'a ForgotPasswordPageContext) -> Result<ForgotPasswordPageContentTemplate<'a>, Box<dyn Error>> {
        let content = create_common_params(context).await?;
        Ok(ForgotPasswordPageContentTemplate { content })
    }
}

fn get_page_title<'a>(content: &ForgotPasswordTemplateCommon<'a>) -> &'a str {
    if content.session.is_empty() {
        "Forgot Password"
    } else {
        "Reset Password"
    }
}

fn get_submit_action<'a>(content: &ForgotPasswordTemplateCommon<'a>) -> String {
    if content.session.is_empty() {
        format!("/forgot-password/")
    } else {
        format!("/password-reset/{}/", content.session)
    }
}

async fn create_common_params<'a>(context: &'a ForgotPasswordPageContext) -> Result<ForgotPasswordTemplateCommon<'a>, Box<dyn Error>> {

    let mut is_reset_success = false;
    let has_session = !context.params.session.is_empty();
    let validation_alert = get_validation_alert(&context.params.validation_report);
    
    if let Some(validation_alert) = &validation_alert {
        if validation_alert.variant == "success" {
            is_reset_success = true;
        }
    } else if has_session {
        if get_password_reset_session_username(&context.params.session).is_none() {
            return Err(
                Box::new(
                    io::Error::new(io::ErrorKind::Other, "Password reset link expired.")
                )
            )
        }
    }

    let captcha = if has_session {
        None
    } else {
        Some(
            CaptchaTemplate {
                captcha_id: generate_captcha()?,
                pow_challenge: String::from(""),
                form_id_prefix: "forgot-password",
            }
        )
    };

    Ok(
        ForgotPasswordTemplateCommon {
            validation_alert,
            captcha,
            username: &context.params.username,
            session: &context.params.session,
            is_reset_success,
        }
    )
}

fn get_validation_alert<'a>(report: &Option<Report>) -> Option<AlertTemplate<'a>> {
    match report {
        Some(report) => {
            let mut message_html: String = "".to_owned();
            let mut variant: &str = "danger";

            if report_has_field(report, "send_success") {
                message_html.push_str(r#"<p>Please check your email for a link to reset your password.</p><p class="mt-3">Allow the email at least 10 minutes to arrive, and check your junk/spam folder if you do not see it.</p>"#);
                variant = "success";
            }
            if report_has_field(report, "reset_success") {
                message_html.push_str("<p>Your password has been updated successfully. You can try signing in in with your new password now.</p>");
                variant = "success";
            }
            if report_has_field(report, "server_error") {
                message_html.push_str("<p>A system error occurred. Please try again later.</p>");
            }
            if report_has_field(report, "no_contact") {
                message_html.push_str("<p>We have no way to contact you in order to send a password reset email.</p>");
            }
            if report_has_field(report, "rate_limit") {
                message_html.push_str("<p>You are allowed to send 5 password reset requests within 24 hours. Please try again later.</p>");
            }
            if report_has_field(report, "username") {
                message_html.push_str("<p><strong>Username:</strong> Invalid username entered.</p>");
            }
            if report_has_field(report, "password") {
                message_html.push_str("<p><strong>Password:</strong> Password must be at least 1 character long and cannot contain invisible characters.</p>");
            }
            if report_has_field(report, "captcha_entry") {
                message_html.push_str("<p><strong>Human Verification:</strong> The correct letters were not entered.</p>");
            }
            if report_has_field(report, "session_expired") {
                message_html.push_str(r#"<p>This link has expired. Please visit the <a href="/forgot-password/">forgot password</a> page to send a new link.</p>"#);
            }

            Some(AlertTemplate {
                variant,
                message_html,
            })
        },
        _ => None,
    }
}
