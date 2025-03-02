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
use crate::ui_pages::chat_box::{ ChatBoxPageTemplate };
use crate::util::captcha::validate_captcha;
use crate::util::rate_limit::rate_limit_exceeded;

#[derive(Default, Debug, RouteParamsContext)]
pub struct ChatBoxPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(default = "")]
    pub comment: String,
}
pub type ChatBoxPageContext = BaseContext<ChatBoxPageParams>;

pub async fn get_chat_box(
    Context { context }: Context<ChatBoxPageParams>,
) -> Response {

    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                _ => render_template!(ChatBoxPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct SubmitChatBoxPageParams {
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
pub async fn post_chat_box(
    Context { context }: Context<SubmitChatBoxPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(ChatBoxPageParams {
        validation_report: None,
        comment: context.params.comment.clone(),
    });

    let is_guest = context.user.is_none();

    if is_guest && context.params.captcha_entry.is_empty() {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("captcha_required"), String::from("Captcha required."))
        );
        return send_chat_box_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let validation_result = validate_chat_box_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_chat_box_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let ip_address_rate_limit_key = format!("chat_box_{}", &context.ip_address);
    if
        rate_limit_exceeded(ip_address_rate_limit_key.as_str(), 10, 60)
    {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("rate_limit"), String::from("Rate limit exceeded."))
        );
        return send_chat_box_page_response(StatusCode::TOO_MANY_REQUESTS, page_context).await;
    }

    let username = if is_guest { "Guest" } else { context.user.as_ref().unwrap().username.as_str() };

    let comment = Comment {
        username: username.to_string(),
        ip_address: Some(context.ip_address.clone()),
        section: CommentSectionName::Chatbox,
        section_tag_id: Some(-1),
        reply_id: -1,
        comment: context.params.comment.clone(),
        ..Comment::default()
    };

    if let Err(error) = database::create_comment(comment).await {
        tracing::warn!("Database call failed when user {} tried to send a chatbox message. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_chat_box_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    let mut redirect_to: String = String::from("/");
    if let Some(redirect_to_attr) = context.route_query.get("redirect-to") {
        redirect_to = decode(redirect_to_attr.as_str()).expect("UTF-8").to_string();
    }
    return Redirect::to(&redirect_to).into_response()
}

async fn validate_chat_box_form(form: &SubmitChatBoxPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

pub async fn send_chat_box_page_response(status: StatusCode, context: ChatBoxPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    _ => render_template!(ChatBoxPageTemplate, &context),
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
