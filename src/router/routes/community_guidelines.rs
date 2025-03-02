use axum::{
    response::{ Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::ui_pages::community_guidelines::{ CommunityGuidelinesTemplate, CommunityGuidelinesContentTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct CommunityGuidelinesPageParams {
}
pub type CommunityGuidelinesPageContext = BaseContext<CommunityGuidelinesPageParams>;

pub async fn get_community_guidelines(
    Context { context }: Context<CommunityGuidelinesPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(CommunityGuidelinesContentTemplate, &context),
                _ => render_template!(CommunityGuidelinesTemplate, &context),
            }
        }
    ).await
}

pub async fn get_community_guidelines_redirect() -> Redirect {
    Redirect::permanent("/community-guidelines/")
}
