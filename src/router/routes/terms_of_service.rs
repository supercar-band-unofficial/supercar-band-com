use axum::{
    response::{ Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::ui_pages::terms_of_service::{ TermsOfServiceTemplate, TermsOfServiceContentTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct TermsOfServicePageParams {
}
pub type TermsOfServicePageContext = BaseContext<TermsOfServicePageParams>;

pub async fn get_terms_of_service(
    Context { context }: Context<TermsOfServicePageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(TermsOfServiceContentTemplate, &context),
                _ => render_template!(TermsOfServiceTemplate, &context),
            }
        }
    ).await
}

pub async fn get_terms_of_service_redirect() -> Redirect {
    Redirect::permanent("/terms-of-service/")
}
