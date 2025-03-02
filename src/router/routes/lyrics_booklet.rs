use axum::{
    response::{ Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::ui_pages::lyrics_booklet::{ LyricsBookletTemplate, LyricsBookletContentTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct LyricsBookletPageParams {
    #[route_param_source(source = "path", name = "band", default = "supercar")]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,
}
pub type LyricsBookletPageContext = BaseContext<LyricsBookletPageParams>;

pub async fn get_lyrics_booklet(
    Context { context }: Context<LyricsBookletPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(LyricsBookletContentTemplate, &context),
                _ => render_template!(LyricsBookletTemplate, &context),
            }
        }
    ).await
}
