use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };
use urlencoding::decode;

use crate::database::{ self, Comment, CommentSectionName };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::router::validation::create_simple_report;
use crate::ui_pages::comment_section::{ CommentSectionPageTemplate };
use crate::util::captcha::validate_captcha;
use crate::util::rate_limit::rate_limit_exceeded;

#[derive(Default, Debug, RouteParamsContext)]
pub struct CommentSectionPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "section", default = "")]
    pub section: String,

    #[route_param_source(source = "path", name = "section_tag_id", default = "-1")]
    pub section_tag_id: i32,

    #[route_param_source(source = "path", name = "reply_id", default = "-1")]
    pub reply_id: i32,

    #[route_param_source(default = "")]
    pub comment: String,
}
pub type CommentSectionPageContext = BaseContext<CommentSectionPageParams>;

pub async fn get_comment_section(
    Context { context }: Context<CommentSectionPageParams>,
) -> Response {

    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                _ => render_template!(CommentSectionPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct SubmitCommentSectionPageParams {
    #[route_param_source(source = "path", name = "section", default = "")]
    #[garde(skip)]
    pub section: String,

    #[route_param_source(source = "path", name = "section_tag_id", default = "-1")]
    #[garde(skip)]
    pub section_tag_id: i32,

    #[route_param_source(source = "path", name = "reply_id", default = "-1")]
    #[garde(skip)]
    pub reply_id: i32,

    #[route_param_source(source = "form", name = "comment", default = "")]
    #[garde(
        length(min = 1, max = 5000)
    )]
    pub comment: String,

    #[route_param_source(source = "form", name = "captcha-id", default = "")]
    #[garde(skip)]
    pub captcha_id: String,

    #[route_param_source(source = "form", name = "captcha-entry", default = "")]
    #[garde(
        custom(is_valid_captcha(&self.captcha_id))
    )]
    pub captcha_entry: String,
}

#[axum::debug_handler]
pub async fn post_comment_section(
    Context { context }: Context<SubmitCommentSectionPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(CommentSectionPageParams {
        validation_report: None,
        section: context.params.section.clone(),
        section_tag_id: context.params.section_tag_id.clone(),
        reply_id: context.params.reply_id.clone(),
        comment: context.params.comment.clone(),
    });

    let is_guest = context.user.is_none();

    let section_parsed = context.params.section.parse::<CommentSectionName>();
    if let Err(_) = section_parsed {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("system_error"), String::from("Bad section."))
        );
        return send_comment_section_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }
    let section = section_parsed.unwrap();

    if is_guest && context.params.captcha_entry.is_empty() {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("captcha_required"), String::from("Captcha required."))
        );
        return send_comment_section_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let validation_result = validate_comment_section_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_comment_section_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let ip_address_rate_limit_key = format!("comment_section_{}", &context.ip_address);
    if rate_limit_exceeded(ip_address_rate_limit_key.as_str(), 10, 60) {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("rate_limit"), String::from("Rate limit exceeded."))
        );
        return send_comment_section_page_response(StatusCode::TOO_MANY_REQUESTS, page_context).await;
    }

    let username = if is_guest { "Guest" } else { context.user.as_ref().unwrap().username.as_str() };

    let comment = Comment {
        username: username.to_string(),
        ip_address: Some(context.ip_address.clone()),
        section,
        section_tag_id: Some(context.params.section_tag_id),
        reply_id: context.params.reply_id,
        comment: context.params.comment.clone(),
        ..Comment::default()
    };

    if let Err(error) = database::create_comment(comment).await {
        tracing::warn!("Database call failed when user {} tried to post a comment. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_comment_section_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    let mut redirect_to: String = String::from("/");
    if let Some(redirect_to_attr) = context.route_query.get("redirect-to") {
        redirect_to = decode(redirect_to_attr.as_str()).expect("UTF-8").to_string();
    }
    return Redirect::to(&redirect_to).into_response()
}

async fn validate_comment_section_form(form: &SubmitCommentSectionPageParams) -> Result<(), Report> {
    if form.reply_id > -1 {
        if let Err(_) = database::get_comment_by_id(form.reply_id).await {
            return Err(
                create_simple_report(String::from("reply_comment_missing"), String::from("Missing comment."))
            );
        }
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

pub async fn send_comment_section_page_response(status: StatusCode, context: CommentSectionPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    _ => render_template!(CommentSectionPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

fn is_valid_captcha(id: &str) -> impl FnOnce(&str, &()) -> garde::Result + '_ {
    move |value, _| {
        if id.is_empty() {
            return Ok(());
        }
        match validate_captcha(id, value) {
            Ok(_) => Ok(()),
            Err(_) => Err(garde::Error::new("Captcha validation failed.")),
        }
    }
}
