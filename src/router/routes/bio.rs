use axum::{
    response::{ Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::ui_pages::bio::{ BioTemplate, BioContentTemplate, BioTabContainerTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct BioPageParams {
    #[route_param_source(source = "query", name = "topic", default = "band")]
    pub topic: String,
}
pub type BioPageContext = BaseContext<BioPageParams>;

pub async fn get_bio(
    Context { context }: Context<BioPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(BioContentTemplate, &context),
                "bio-tab-container" => render_template!(BioTabContainerTemplate, &context),
                _ => render_template!(BioTemplate, &context),
            }
        }
    ).await
}

pub async fn get_bio_redirect() -> Redirect {
    Redirect::permanent("/bio/")
}
