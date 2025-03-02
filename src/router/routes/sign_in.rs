use axum::{
    http::StatusCode,
    http::header::HeaderValue,
    response::{ IntoResponse, Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };
use tokio::task;
use urlencoding::decode;

use crate::database;
use crate::ui_modules::account_summary::{ AccountSummaryTemplate, AccountSummaryParams };
use crate::ui_pages::sign_in::{ SignInTemplate, SignInContentTemplate };
use crate::router::{ get_hx_target, html_to_response };
use crate::router::authn::{ AuthSession, Credentials, UserSession };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct SignInPageParams {
    #[route_param_source(default = "0")]
    pub status: u16,

    #[route_param_source(default = "")]
    pub entered_username: String,
}
pub type SignInPageContext = BaseContext<SignInPageParams>;

pub async fn get_sign_in(
    Context { context }: Context<SignInPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(SignInContentTemplate, &context),
                _ => render_template!(SignInTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, RouteParamsContext)]
pub struct SignInRequestParams {
    #[route_param_source(source="form")]
    pub username: String,

    #[route_param_source(source="form")]
    pub password: String,
}

pub async fn post_sign_in(
    mut auth_session: AuthSession,
    Context { context: request_context }: Context<SignInRequestParams>,
) -> Response {
    let mut status: StatusCode = StatusCode::OK;

    let credentials = Credentials {
        username: request_context.params.username.clone(),
        password: request_context.params.password.clone(),
        ip_address: request_context.ip_address.clone(),
    };
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
        status = StatusCode::INTERNAL_SERVER_ERROR;
    }

    let mut context = request_context.clone_with_params(
        SignInPageParams {
            status: status.as_u16(),
            entered_username: request_context.params.username.clone(),
        }
    );
    let hx_target = get_hx_target(&context.route_headers);
    if status != StatusCode::OK {
        return (
            status,
            html_to_response(
                &context,
                |hx_target, context| async move {
                    match hx_target.as_str() {
                        "site-account-summary" => render_template!(SignInContentTemplate, &context),
                        _ => render_template!(SignInTemplate, &context),
                    }
                }
            ).await
        ).into_response();
    }

    let update_login_time_username = user.username.clone();
    task::spawn(async move {
        let _ = database::update_user_at_login(
            &update_login_time_username,
            &request_context.ip_address,
        ).await;
    });

    if hx_target.is_empty() {
        let mut redirect_to: String = String::from("/");
        if let Some(redirect_to_attr) = context.route_query.get("redirect-to") {
            redirect_to = decode(redirect_to_attr.as_str()).expect("UTF-8").to_string();
        }
        return Redirect::to(&redirect_to).into_response()
    }

    context.user = Some(user);

    let mut response = html_to_response(
        &context,
        |_, context| async move {
            render_template!(AccountSummaryTemplate, AccountSummaryParams { context: &context })
        }
    ).await;

    response.headers_mut().insert(
        "HX-Refresh",
        HeaderValue::from_static("true"),
    );

    response
}
