use axum::{
    http::header::HeaderValue,
    response::{ IntoResponse, Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };
use urlencoding::decode;

use crate::ui_pages::sign_in::{ SignInTemplate, SignInContentTemplate };
use crate::router::{ get_hx_target, html_to_response };
use crate::router::context::{ Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct SignOutRequestParams {}

use crate::router::authn::{ AuthSession };
use crate::router::routes::sign_in::SignInPageParams;

pub async fn post_sign_out(
    mut auth_session: AuthSession,
    Context { context: request_context }: Context<SignOutRequestParams>,
) -> Response {

    match auth_session.logout().await {
        Ok(_) => (),
        Err(_) => (),
    }

    let context = request_context.clone_with_params(
        SignInPageParams {
            status: 0,
            entered_username: String::from(""),
        }
    );

    let hx_target = get_hx_target(&context.route_headers);
    if hx_target.is_empty() {
        let mut redirect_to: String = String::from("/");
        if let Some(redirect_to_attr) = context.route_query.get("redirect-to") {
            redirect_to = decode(redirect_to_attr.as_str()).expect("UTF-8").to_string();
        }
        return Redirect::to(&redirect_to).into_response()
    }

    let mut response = html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "site-account-summary" => render_template!(SignInContentTemplate, &context),
                _ => render_template!(SignInTemplate, &context),
            }
        }
    ).await;

    response.headers_mut().insert(
        "HX-Refresh",
        HeaderValue::from_static("true"),
    );

    response
}
