use std::error::Error;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, UserPermission };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::delete_lyrics::{ DeleteLyricsPageTemplate, DeleteLyricsPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeleteLyricsPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    pub song: String,

    #[route_param_source(source = "path", name = "contributor", default = "")]
    pub contributor: String,
}
pub type DeleteLyricsPageContext = BaseContext<DeleteLyricsPageParams>;

pub async fn get_delete_lyrics(
    Context { mut context }: Context<DeleteLyricsPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::DeleteOwnLyrics)
            || user.permissions.contains(&UserPermission::DeleteLyrics),
        None => false,
    };
    if !has_permissions {
        context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
    }

    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(DeleteLyricsPageContentTemplate, &context),
                _ => render_template!(DeleteLyricsPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmLyricsPageParams {
    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    #[garde(skip)]
    pub song: String,

    #[route_param_source(source = "path", name = "contributor", default = "")]
    #[garde(skip)]
    pub contributor: String,
}

#[axum::debug_handler]
pub async fn delete_lyrics(
    Context { context }: Context<DeleteConfirmLyricsPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeleteLyricsPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        album: context.params.album.clone(),
        song: context.params.song.clone(),
        contributor: context.params.contributor.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeleteLyrics) || (
                user.permissions.contains(&UserPermission::DeleteOwnLyrics)
                && user.username == context.params.contributor
            )
        },
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_lyrics_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_lyrics_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_lyrics_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let lyrics_id = validation_result.unwrap();
    let username = context.user.unwrap().username;

    if let Err(error) = database::mark_lyrics_for_deletion(lyrics_id).await {
        tracing::warn!("Database call failed when user {} tried to delete lyrics. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_lyrics_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/lyrics/{}/{}/", context.params.band, context.params.album).as_str()
    ).into_response()
}

async fn validate_lyrics_delete_form(form: &DeleteConfirmLyricsPageParams) -> Result<i32, Report> {
    let lyrics_id = validate_lyrics_exists(&form.band, &form.song, &form.contributor).await;
    if let Err(_) = lyrics_id {
        return Err(
            create_simple_report(String::from("song_missing"), String::from("The specified song does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(lyrics_id.unwrap())
}

async fn validate_lyrics_exists(band_slug: &str, song_slug: &str, contributor: &str) -> Result<i32, Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    let song = database::get_song_by_slug_and_band_id(song_slug, band.id).await?;
    let lyrics = database::get_lyrics_by_username_and_song_id(contributor, song.id).await?;
    Ok(lyrics.id)
}

pub async fn send_delete_lyrics_page_response(status: StatusCode, context: DeleteLyricsPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeleteLyricsPageContentTemplate, &context),
                    _ => render_template!(DeleteLyricsPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
