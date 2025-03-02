use axum::{
    response::{ Response, Redirect },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self };
use crate::ui_pages::photos::{ PhotosTemplate, PhotosContentTemplate, PhotosCommentsTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct PhotosPageParams {
    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "path", name = "photo", default = "-1")]
    pub photo: i32,

    #[route_param_source(source = "query", name = "comments-page", default = "1")]
    pub comments_page: u32,
}
pub type PhotosPageContext = BaseContext<PhotosPageParams>;

pub async fn get_photos(
    Context { context }: Context<PhotosPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(PhotosContentTemplate, &context),
                "page-comments" => render_template!(PhotosCommentsTemplate, &context),
                _ => render_template!(PhotosTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, RouteParamsContext)]
pub struct PhotosRedirectParams {
    #[route_param_source(source = "query", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "query", name = "photo", default = "-1")]
    pub photo: i32,
}

pub async fn get_photos_redirect(
    Context { context }: Context<PhotosRedirectParams>,
) -> Redirect {
    if !context.params.album.is_empty() {
        let album_slug = match database::get_photo_album_by_title(&context.params.album).await {
            Ok(result) => result.slug,
            Err(_) => String::from("*"),
        };

        if context.params.photo > -1 {
            return Redirect::permanent(
                &format!("/photos/{}/{}/", &album_slug, &context.params.photo)
            );
        }
        return Redirect::permanent(
            &format!("/photos/{}/", &album_slug)
        );
    }
    Redirect::permanent(
        &format!("/photos/")
    )
}
