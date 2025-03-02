use axum::{
    http::StatusCode,
    http::header::HeaderValue,
    response::{ Redirect, Response, IntoResponse },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database;
use crate::ui_pages::sign_up::{ SignUpTemplate, SignUpContentTemplate };
use crate::router::{ html_to_response };
use crate::router::authn::{ AuthSession, Credentials, UserSession };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::util::captcha::{ validate_captcha };
use crate::util::rate_limit::rate_limit_exceeded;
use crate::util::user::create_user_profile_href;
use crate::router::validation::create_simple_report;

#[derive(Default, RouteParamsContext)]
pub struct SignUpPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "none", default = "")]
    pub username: String,

    #[route_param_source(source = "none", default = "")]
    pub first_name: String,

    #[route_param_source(source = "none", default = "")]
    pub last_name: String,
}
pub type SignUpPageContext = BaseContext<SignUpPageParams>;

pub async fn get_sign_up(
    Context { context }: Context<SignUpPageParams>,
) -> Response {
    send_sign_up_page_response(StatusCode::OK, context).await
}

pub async fn send_sign_up_page_response(status: StatusCode, context: SignUpPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(SignUpContentTemplate, &context),
                    _ => render_template!(SignUpTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

pub async fn get_sign_up_redirect() -> Redirect {
    Redirect::permanent("/sign-up/")
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct SignUpRequestParams {
    #[route_param_source(source = "form", name = "username", default = "")]
    #[garde(
        length(min = 1, max = 30),
        // See src/util/regex.txt
        pattern(r"^[\u0020-\u0029\u002B-\u002E\u0030-\u0039\u003B-\u005B\u005D-\u007E\u0080\u0082-\u008C\u008E\u0091-\u009C\u009E-\u009F\u00A1-\u00AC\u00AE-\u00FF\u0100-\u024F\u0250-\u02AF\u02B0-\u02FF\u0300-\u036F\u0370-\u03FF\u0400-\u04FF\u2150-\u218F\u2190-\u21FF\u2200-\u22FF\u2300-\u23FF\u2460-\u24FF\u2500-\u257F\u2E80-\u2EFF\u2F00-\u2FDF\u3000-\u303F\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF\uF900-\uFAFF\u3400-\u4DBF\u{20000}-\u{2A6DF}\u{2A700}-\u{2B73F}\u{2B740}-\u{2B81F}\u{2B820}-\u{2CEAF}\u{2CEB0}-\u{2EBEF}\u{2F800}-\u{2FA1F}\u{30000}-\u{3134F}]{1,30}$"),
    )]
    pub username: String,

    #[route_param_source(source = "form", name = "first-name", default = "")]
    #[garde(
        length(min = 1, max = 30),
    )]
    pub first_name: String,

    #[route_param_source(source = "form", name = "last-name", default = "")]
    #[garde(
        length(max = 30),
    )]
    pub last_name: String,

    #[route_param_source(source = "form", name = "password", default = "")]
    #[garde(
        length(min = 1),
        // All visible characters allowed
        pattern(r"^[\x20-\x7E\xA0-\u10FFFF]*$")
    )]
    pub password: String,

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
pub async fn post_sign_up(
    mut auth_session: AuthSession,
    Context { context }: Context<SignUpRequestParams>,
) -> Response {
    let mut page_context = context.clone_with_params(SignUpPageParams {
        validation_report: None,
        username: context.params.username.clone(),
        first_name: context.params.first_name.clone(),
        last_name: context.params.last_name.clone(),
    });

    if let Err(report) = validate_sign_up(&context.params).await {
        page_context.params.validation_report = Some(report);
        return send_sign_up_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let ip_address_rate_limit_key = format!("sign_up_{}", &context.ip_address);
    if rate_limit_exceeded(ip_address_rate_limit_key.as_str(), 2, 86400) {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("rate_limit"), String::from("Rate limit exceeded."))
        );
        return send_sign_up_page_response(StatusCode::TOO_MANY_REQUESTS, page_context).await;
    }

    let new_user = database::User {
        username: context.params.username.trim().to_string(),
        password: context.params.password.clone(),
        first_name: context.params.first_name.clone(),
        last_name: context.params.last_name.clone(),
        ip_address: context.ip_address.clone(),
        ..Default::default()
    };
    if let Err(error) = database::create_user(new_user).await {
        tracing::warn!("Database call failed when a guest tried to sign up. {:?}", error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_sign_up_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    let _ = database::notify_user_registered(&context.params.username).await;

    let credentials = Credentials {
        username: context.params.username.clone(),
        password: context.params.password.clone(),
        ip_address: context.ip_address.clone(),
    };
    let mut status: StatusCode = StatusCode::OK;
    let user = match auth_session.authenticate(credentials).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            status = StatusCode::UNAUTHORIZED;
            UserSession::default()
        },
        Err(_) => {
            status = StatusCode::INTERNAL_SERVER_ERROR;
            UserSession::default()
        },
    };

    if status == StatusCode::OK && auth_session.login(&user).await.is_err() {
        tracing::warn!("Failed to sign in user {} after registration.", &context.params.username);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_sign_up_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    let mut response = Redirect::to(
        format!("{}?first-visit=true", create_user_profile_href(&context.params.username)).as_str()
    ).into_response();
    response.headers_mut().insert(
        "HX-Refresh",
        HeaderValue::from_static("true"),
    );

    response
}

async fn validate_sign_up(form: &SignUpRequestParams) -> Result<(), Report> {
    if let Err(_) = validate_username_not_taken(&form.username).await {
        return Err(
            create_simple_report(String::from("username_taken"), String::from("Username is already taken."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

async fn validate_username_not_taken(username: &str) -> Result<(), ()> {
    let lowercase_username = username.trim().to_lowercase();
    if lowercase_username == "guest" || lowercase_username == "admin" || lowercase_username == "administrator" {
        return Err(());
    }
    match database::get_user_by_username(username).await {
        Ok(_) => Err(()),
        Err(_) => Ok(()),
    }
}

fn is_valid_captcha(id: &str) -> impl FnOnce(&str, &()) -> garde::Result + '_ {
    move |value, _| {
        match validate_captcha(id, value) {
            Ok(_) => Ok(()),
            Err(_) => Err(garde::Error::new("Captcha validation failed.")),
        }
    }
}
