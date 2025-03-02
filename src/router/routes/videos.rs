use axum::{
    response::{ Response, Redirect },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self };
use crate::ui_pages::videos::{ VideosTemplate, VideosContentTemplate, VideosCommentsTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct VideosPageParams {
    #[route_param_source(source = "path", name = "category", default = "")]
    pub category: String,

    #[route_param_source(source = "path", name = "video", default = "")]
    pub video: String,

    #[route_param_source(source = "query", name = "comments-page", default = "1")]
    pub comments_page: u32,
}
pub type VideosPageContext = BaseContext<VideosPageParams>;

pub async fn get_videos(
    Context { context }: Context<VideosPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(VideosContentTemplate, &context),
                "page-comments" => render_template!(VideosCommentsTemplate, &context),
                _ => render_template!(VideosTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, RouteParamsContext)]
pub struct VideosRedirectParams {
    #[route_param_source(source = "query", name = "category", default = "-1")]
    pub category: i32,

    #[route_param_source(source = "query", name = "video", default = "-1")]
    pub video: i32,
}

pub async fn get_videos_redirect(
    Context { context }: Context<VideosRedirectParams>,
) -> Redirect {
    if context.params.category > -1 {
        let category_slug = match database::get_video_category_by_id(context.params.category).await {
            Ok(result) => result.slug,
            Err(_) => String::from("*"),
        };

        if context.params.video > -1 {
            let video_slug = match database::get_video_by_id(context.params.video).await {
                Ok(result) => result.slug,
                Err(_) => String::from("*"),
            };
            return Redirect::permanent(
                &format!("/videos/{}/{}/", &category_slug, &video_slug)
            );
        }
        return Redirect::permanent(
            &format!("/videos/{}/", &category_slug)
        );
    }
    Redirect::permanent(
        &format!("/videos/")
    )
}
