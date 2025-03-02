use axum::{
    response::{ Redirect, Response },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };

use crate::database;
use crate::ui_pages::lyrics::{ LyricsTemplate, LyricsContentTemplate, LyricsCommentsTemplate };
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::util::format::to_kebab_case;

#[derive(Default, RouteParamsContext)]
pub struct LyricsPageParams {
    #[route_param_source(source = "path", name = "band", default = "supercar")]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    pub song: String,

    #[route_param_source(source = "query", name = "contributor", default = "")]
    pub contributor: String,

    #[route_param_source(source = "query", name = "search", default = "")]
    pub search: String,

    #[route_param_source(source = "query", name = "comments-page", default = "1")]
    pub comments_page: u32,
}
pub type LyricsPageContext = BaseContext<LyricsPageParams>;

pub async fn get_lyrics(
    Context { context }: Context<LyricsPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(LyricsContentTemplate, &context),
                "page-comments" => render_template!(LyricsCommentsTemplate, &context),
                _ => render_template!(LyricsTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, RouteParamsContext)]
pub struct LyricsRedirectParams {
    #[route_param_source(source = "query", name = "band", default = "supercar")]
    pub band: String,

    #[route_param_source(source = "query", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "query", name = "song", default = "")]
    pub song: String,
}

pub async fn get_lyrics_redirect(
    Context { context }: Context<LyricsRedirectParams>,
) -> Redirect {
    let band: String = to_kebab_case(&context.params.band);
    if !context.params.song.is_empty() {
        let song = to_kebab_case(&context.params.song);
        let album = if context.params.album.is_empty() {
            match database::get_album_by_song_slug(&song, &band).await {
                Ok(album) => album.album_slug,
                Err(_) => String::from("*")
            }
        } else {
            to_kebab_case(&context.params.album)
        };
        return Redirect::permanent(
            &format!("/lyrics/{}/{}/{}/", &band, &album, &song)
        );
    } else if !context.params.album.is_empty() {
        return Redirect::permanent(
            &format!("/lyrics/{}/{}/", &band, to_kebab_case(&context.params.album))
        );
    }
    Redirect::permanent(
        &format!("/lyrics/{}/", &band)
    )
}
