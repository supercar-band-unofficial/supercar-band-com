use axum::{
    response::{ Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::ui_pages::privacy_policy::{ PrivacyPolicyTemplate, PrivacyPolicyContentTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct PrivacyPolicyPageParams {
}
pub type PrivacyPolicyPageContext = BaseContext<PrivacyPolicyPageParams>;

pub async fn get_privacy_policy(
    Context { context }: Context<PrivacyPolicyPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(PrivacyPolicyContentTemplate, &context),
                _ => render_template!(PrivacyPolicyTemplate, &context),
            }
        }
    ).await
}

pub async fn get_privacy_policy_redirect() -> Redirect {
    Redirect::permanent("/privacy-policy/")
}
