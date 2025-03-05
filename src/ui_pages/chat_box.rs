use std::error::Error;
use askama::Template;
use garde::{ Report };
use urlencoding::encode;

use crate::router::routes::chat_box::ChatBoxPageContext;
use crate::router::validation::report_has_field;
use crate::ui_primitives::alert::AlertTemplate;
use crate::ui_primitives::captcha::{ CaptchaTemplate };
use crate::util::captcha::{ generate_captcha };

struct ChatBoxPageTemplateCommon<'a> {
    validation_alert: Option<AlertTemplate<'a>>,
    allow_submit: bool,
    captcha: CaptchaTemplate<'a>,
    comment: &'a str,
    redirect_url: String,
    redirect_url_encoded: String,
}

#[derive(Template)]
#[template(path = "ui_pages/chat_box.html")]
pub struct ChatBoxPageTemplate<'a> {
    active_page: &'a str,
    content: ChatBoxPageTemplateCommon<'a>,
}
impl<'a> ChatBoxPageTemplate<'a> {
    pub async fn new(context: &'a ChatBoxPageContext) -> Result<ChatBoxPageTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";

        let content = create_common_params(context).await?;

        Ok(ChatBoxPageTemplate { active_page, content })
    }
}

async fn create_common_params<'a>(context: &'a ChatBoxPageContext) -> Result<ChatBoxPageTemplateCommon<'a>, Box<dyn Error>> {
    let validation_alert = get_validation_alert(&context.params.validation_report);

    let mut allow_submit = true;
    if let Some(report) = &context.params.validation_report {
        allow_submit = !report_has_field(report, "comment");
    }

    let captcha = CaptchaTemplate {
        captcha_id: generate_captcha()?,
        pow_challenge: String::from(""),
        form_id_prefix: "chat-box",
    };

    let current_url = context.route_original_uri.to_string();
    let redirect_url = context.route_query
        .get("redirect-to")
        .unwrap_or(&current_url);

    Ok(
        ChatBoxPageTemplateCommon {
            validation_alert,
            allow_submit,
            captcha,
            comment: &context.params.comment,
            redirect_url: redirect_url.to_string(),
            redirect_url_encoded: encode(redirect_url).to_string(),
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
            if report_has_field(report, "comment") {
                message_html.push_str("<p>The comment you entered is either too long or contains invalid characters.</p>");
            }
            if report_has_field(report, "captcha_required") {
                message_html.push_str("<p>Please fill out the captcha.</p>");
            }
            if report_has_field(report, "captcha_entry") {
                message_html.push_str("<p><strong>Human Verification:</strong> The correct letters were not entered.</p>");
            }
            if report_has_field(report, "rate_limit") {
                message_html.push_str("<p>You are submitting messages too quickly.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
