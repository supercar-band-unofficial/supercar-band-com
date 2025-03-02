use axum::{
    response::{ Response, Redirect },
};
use askama::Template;
use macros::{ RouteParamsContext, render_template };
use urlencoding::encode;

use crate::database::{ self, Band, Song, SongTab };
use crate::ui_pages::tabs::{ TabsTemplate, TabsContentTemplate, TabsCommentsTemplate };
use crate::util::format::to_kebab_case;
use crate::router::{ html_to_response };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };

#[derive(Default, RouteParamsContext)]
pub struct TabsPageParams {
    #[route_param_source(source = "path", name = "band", default = "supercar")]
    pub band: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    pub song: String,

    #[route_param_source(source = "path", name = "tab_type", default = "")]
    pub tab_type: String,

    #[route_param_source(source = "path", name = "contributor", default = "")]
    pub contributor: String,

    #[route_param_source(source = "query", name = "comments-page", default = "1")]
    pub comments_page: u32,
}
pub type TabsPageContext = BaseContext<TabsPageParams>;

pub async fn get_tabs(
    Context { context }: Context<TabsPageParams>,
) -> Response {
    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(TabsContentTemplate, &context),
                "page-comments" => render_template!(TabsCommentsTemplate, &context),
                _ => render_template!(TabsTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, RouteParamsContext)]
pub struct TabsRedirectParams {
    #[route_param_source(source = "query", name = "band", default = "supercar")]
    pub band: String,

    #[route_param_source(source = "query", name = "song", default = "")]
    pub song: String,

    #[route_param_source(source = "query", name = "tab", default = "")]
    pub tab: String,
}

pub async fn get_tabs_redirect(
    Context { context }: Context<TabsRedirectParams>,
) -> Redirect {
    let band_slug: String = to_kebab_case(&context.params.band);
    if !context.params.tab.is_empty() {
        let tab_id = context.params.tab.parse::<i32>().unwrap_or_else(|_| 0);

        let tab = database::get_song_tab_by_id(tab_id).await.unwrap_or_else(|_| SongTab::default());
        let song = database::get_song_by_id(tab.song).await.unwrap_or_else(|_| Song::default());
        let band = database::get_band_by_id(song.band).await.unwrap_or_else(|_| Band::default());

        return Redirect::permanent(
            &format!("/tabs/{}/{}/{}/{}/", &band.band_slug, &song.song_slug, to_kebab_case(tab.tab_type.to_string().as_str()), encode(&tab.username))
        );
    } else if !context.params.song.is_empty() {
        let song_slug = to_kebab_case(&context.params.song);
        return Redirect::permanent(
            &format!("/tabs/{}/{}/", &band_slug, &song_slug)
        );
    }
    Redirect::permanent(
        &format!("/tabs/{}/", &band_slug)
    )
}
