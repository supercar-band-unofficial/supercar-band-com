use std::error::Error;
use std::io;
use askama::Template;
use garde::{ Report };
use urlencoding::encode;

use crate::database::{ self, Comment };
use crate::router::routes::comment_section::CommentSectionPageContext;
use crate::router::validation::report_has_field;
use crate::ui_primitives::alert::AlertTemplate;
use crate::ui_primitives::captcha::{ CaptchaTemplate };
use crate::util::captcha::{ generate_captcha };
use crate::util::user::{ is_guest_user };

struct CommentSectionPageTemplateCommon<'a> {
    validation_alert: Option<AlertTemplate<'a>>,
    allow_submit: bool,
    username: String,
    profile_picture_filename: String,
    section: String,
    section_tag_id: i32,
    reply_id: i32,
    reply_to_comment: Option<Comment>,
    captcha: Option<CaptchaTemplate<'a>>,
    comment: &'a str,
    redirect_url: String,
    redirect_url_encoded: String,
}

#[derive(Template)]
#[template(path = "ui_pages/comment_section.html")]
pub struct CommentSectionPageTemplate<'a> {
    active_page: &'a str,
    content: CommentSectionPageTemplateCommon<'a>,
}
impl<'a> CommentSectionPageTemplate<'a> {
    pub async fn new(context: &'a CommentSectionPageContext) -> Result<CommentSectionPageTemplate<'a>, Box<dyn Error>> {
        let active_page: &str = "";

        let content = create_common_params(context).await?;

        Ok(CommentSectionPageTemplate { active_page, content })
    }
}

async fn create_common_params<'a>(context: &'a CommentSectionPageContext) -> Result<CommentSectionPageTemplateCommon<'a>, Box<dyn Error>> {
    let validation_alert = get_validation_alert(&context.params.validation_report);

    let mut allow_submit = true;
    if let Some(report) = &context.params.validation_report {
        allow_submit = !report_has_field(report, "comment");
    }

    let mut username = String::from("Guest");
    let profile_picture_filename = if let Some(user) = &context.user {
        let user = database::get_user_by_username(&user.username).await?;
        username = user.username.clone();
        user.profile_picture_filename
    } else {
        String::from("Guest.jpeg")
    };

    let captcha = if is_guest_user(&username) {
        Some(
            CaptchaTemplate {
                captcha_id: generate_captcha()?,
                form_id_prefix: "chat-box",
            }
        )
    } else {
        None
    };

    let reply_to_comment = if context.params.reply_id > -1 {
        Some(database::get_comment_by_id(context.params.reply_id).await?)
    } else {
        None
    };

    if let Some(reply_to_comment) = &reply_to_comment {
        if reply_to_comment.section.to_string() != context.params.section
            || reply_to_comment.section_tag_id.unwrap_or_else(|| -1) != context.params.section_tag_id {
            return Err(
                Box::new(
                    io::Error::new(io::ErrorKind::Other, "Slug was too long.")
                )
            );
        }
    }

    let current_url = context.route_original_uri.to_string();
    let redirect_url = context.route_query
        .get("redirect-to")
        .unwrap_or(&current_url);

    Ok(
        CommentSectionPageTemplateCommon {
            validation_alert,
            allow_submit,
            username: username,
            profile_picture_filename,
            section: context.params.section.clone(),
            section_tag_id: context.params.section_tag_id,
            reply_id: context.params.reply_id,
            reply_to_comment,
            captcha,
            comment: &context.params.comment,
            redirect_url: redirect_url.to_string(),
            redirect_url_encoded: encode(redirect_url).to_string(),
        }
    )
}

fn get_submit_action<'a>(content: &CommentSectionPageTemplateCommon) -> String {
    format!("/comment-section/{}/{}/{}/?redirect-to={}",
        content.section,
        content.section_tag_id,
        content.reply_id,
        content.redirect_url_encoded,
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
            if report_has_field(report, "reply_comment_missing") {
                message_html.push_str("<p>The comment you are replying to doesn't exist. Maybe it was deleted?</p>");
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
