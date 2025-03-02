use axum::{
    response::{ Response, Redirect },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::ui_pages::members::{ MembersTemplate, MembersContentTemplate, MembersCommentsTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct MembersPageParams {
    #[route_param_source(source = "path", name = "username", default = "")]
    pub username: String,

    #[route_param_source(source = "query", name = "members-page", default = "1")]
    pub members_page: u32,

    #[route_param_source(source = "query", name = "comments-page", default = "1")]
    pub comments_page: u32,
}
pub type MembersPageContext = BaseContext<MembersPageParams>;

pub async fn get_members(
    Context { context }: Context<MembersPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(MembersContentTemplate, &context),
                "page-comments" => render_template!(MembersCommentsTemplate, &context),
                _ => render_template!(MembersTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, RouteParamsContext)]
pub struct MembersRedirectParams {
    #[route_param_source(source = "query", name = "member", default = "")]
    pub member: String,
}

pub async fn get_members_redirect(
    Context { context }: Context<MembersRedirectParams>,
) -> Redirect {
    if !context.params.member.is_empty() {
        return Redirect::permanent(
            &format!("/members/{}/", &context.params.member)
        );
    }
    Redirect::permanent(
        &format!("/members/")
    )
}
