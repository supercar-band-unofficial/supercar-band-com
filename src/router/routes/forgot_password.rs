use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response },
};
use askama::Template;
use garde::{ Validate, Report };
use lettre::message::header::ContentType;
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, User };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::router::validation::create_simple_report;
use crate::ui_pages::forgot_password::{ ForgotPasswordPageTemplate, ForgotPasswordPageContentTemplate };
use crate::util::captcha::validate_captcha;
use crate::util::password_reset_session::{ create_password_reset_session, discard_password_reset_session, get_password_reset_session_username };
use crate::util::rate_limit::rate_limit_exceeded;
use crate::util::smtp;

#[derive(Default, Debug, RouteParamsContext)]
pub struct ForgotPasswordPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "session", default = "")]
    pub session: String,

    #[route_param_source(default = "")]
    pub username: String,
}
pub type ForgotPasswordPageContext = BaseContext<ForgotPasswordPageParams>;

pub async fn get_forgot_password(
    Context { context }: Context<ForgotPasswordPageParams>,
) -> Response {

    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(ForgotPasswordPageContentTemplate, &context),
                _ => render_template!(ForgotPasswordPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct SubmitForgotPasswordPageParams {
    #[route_param_source(source = "form", name = "username", default = "")]
    #[garde(
        length(min = 1, max = 30)
    )]
    pub username: String,

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
pub async fn post_forgot_password(
    Context { context }: Context<SubmitForgotPasswordPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(ForgotPasswordPageParams {
        validation_report: None,
        username: context.params.username.clone(),
        session: String::from(""),
    });

    let validation_result = validate_forgot_password_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_forgot_password_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let user = validation_result.unwrap();

    if user.email.is_empty() {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("no_contact"), String::from("No email exists."))
        );
        return send_forgot_password_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let username_rate_limit_key = format!("forgot_password_{}", &user.username);
    let ip_address_rate_limit_key = format!("forgot_password_{}", &context.ip_address);
    if
        rate_limit_exceeded(username_rate_limit_key.as_str(), 5, 86400)
        || rate_limit_exceeded(ip_address_rate_limit_key.as_str(), 5, 86400)
    {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("rate_limit"), String::from("Rate limit exceeded."))
        );
        return send_forgot_password_page_response(StatusCode::TOO_MANY_REQUESTS, page_context).await;
    }

    let password_reset_session_id = create_password_reset_session(user.username.clone());
    let password_reset_link = format!("https://supercarband.com/password-reset/{}/", password_reset_session_id);

    if let Err(error) = smtp::send_email(
        &user.email,
        String::from("Password Reset"),
        format!(
            r#"<p>Please follow the link below to reset your password for supercarband.com:</p>
            <p><a href="{}">{}</a></p>
            <p>This link will be available for 30 minutes. If you did not request a password reset it is safe to ignore this email.</p>"#,
            password_reset_link,
            password_reset_link,
        ),
        ContentType::TEXT_HTML,
    ) {
        tracing::warn!("Database call failed when user {} tried to send a password reset email. {:?}", &user.username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_forgot_password_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    tracing::info!("Password reset link generated by {} for user {}", &context.ip_address, &user.username);

    page_context.params.validation_report = Some(
        create_simple_report(String::from("send_success"), String::from("Reset request success."))
    );
    return send_forgot_password_page_response(StatusCode::OK, page_context).await;
}

async fn validate_forgot_password_form(form: &SubmitForgotPasswordPageParams) -> Result<User, Report> {
    let account_exists_result = validate_account_exists(form.username.as_str()).await;
    if let Err(_) = account_exists_result {
        return Err(
            create_simple_report(String::from("username"), String::from("User does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(account_exists_result.unwrap())
}

async fn validate_account_exists(username: &str) -> Result<User, ()> {
    match database::get_user_by_username(username).await {
        Ok(user) => Ok(user),
        Err(_) => {
            Err(())
        },
    }
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct SubmitResetPasswordPageParams {
    #[route_param_source(source = "path", name = "session", default = "")]
    #[garde(skip)]
    pub session: String,

    #[route_param_source(source = "form", name = "password", default = "")]
    #[garde(
        length(min = 1),
        // All visible characters allowed
        pattern(r"^[\x20-\x7E\xA0-\u10FFFF]*$")
    )]
    pub password: String,
}

#[axum::debug_handler]
pub async fn post_reset_password(
    Context { context }: Context<SubmitResetPasswordPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(ForgotPasswordPageParams {
        validation_report: None,
        username: String::from(""),
        session: context.params.session.clone(),
    });

    let validation_result = validate_reset_password_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_forgot_password_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let user = validation_result.unwrap();

    if let Err(error) = database::update_user_password(
        &user.username,
        &context.params.password
    ).await {
        tracing::warn!("Database call failed when user {} tried to reset their password. {:?}", &user.username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_forgot_password_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    discard_password_reset_session(&context.params.session);

    page_context.params.validation_report = Some(
        create_simple_report(String::from("reset_success"), String::from("Reset request success."))
    );
    return send_forgot_password_page_response(StatusCode::OK, page_context).await;
}

async fn validate_reset_password_form(form: &SubmitResetPasswordPageParams) -> Result<User, Report> {
    let username_option = get_password_reset_session_username(&form.session);
    if username_option.is_none() {
        return Err(
            create_simple_report(String::from("session_expired"), String::from("Link expired."))
        );
    }
    let username = username_option.unwrap();
    let account_exists_result = validate_account_exists(username.as_str()).await;
    if let Err(_) = account_exists_result {
        return Err(
            create_simple_report(String::from("server_error"), String::from("User not found."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(account_exists_result.unwrap())
}

pub async fn send_forgot_password_page_response(status: StatusCode, context: ForgotPasswordPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(ForgotPasswordPageContentTemplate, &context),
                    _ => render_template!(ForgotPasswordPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

fn is_valid_captcha(id: &str) -> impl FnOnce(&str, &()) -> garde::Result + '_ {
    move |value, _| {
        match validate_captcha(id, value) {
            Ok(_) => Ok(()),
            Err(_) => Err(garde::Error::new("Captcha validation failed.")),
        }
    }
}
