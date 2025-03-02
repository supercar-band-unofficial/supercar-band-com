use std::error::Error;
use std::io;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, Lyrics, UserPermission, UserPreference };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_lyrics::{ EditLyricsPageTemplate, EditLyricsPageContentTemplate, EditLyricsSelectBandAlbumSongTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditLyricsPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    pub song: String,

    #[route_param_source(default = "")]
    pub kanji: String,

    #[route_param_source(default = "")]
    pub romaji: String,

    #[route_param_source(default = "")]
    pub english: String,

    #[route_param_source(default = "")]
    pub comment: String,
}
pub type EditLyricsPageContext = BaseContext<EditLyricsPageParams>;

pub async fn get_edit_lyrics(
    Context { mut context }: Context<EditLyricsPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnLyrics)
            || user.permissions.contains(&UserPermission::EditOwnLyrics)
            || user.permissions.contains(&UserPermission::EditLyrics),
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
                "main-article" => render_template!(EditLyricsPageContentTemplate, &context),
                "edit-lyrics-select-song-album-song-section" => render_template!(EditLyricsSelectBandAlbumSongTemplate, &context),
                _ => render_template!(EditLyricsPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreateLyricsPageParams {
    #[route_param_source(source = "form", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "form", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "form", name = "song", default = "")]
    #[garde(skip)]
    pub song: String,

    #[route_param_source(source = "form", name = "kanji", default = "")]
    #[garde(
        length(min = 1, max = 32000),
    )]
    pub kanji: String,

    #[route_param_source(source = "form", name = "romaji", default = "")]
    #[garde(
        length(min = 1, max = 4000),
    )]
    pub romaji: String,

    #[route_param_source(source = "form", name = "english", default = "")]
    #[garde(
        length(min = 1, max = 4000),
    )]
    pub english: String,

    #[route_param_source(source = "form", name = "comment", default = "")]
    #[garde(
        length(max = 2000),
    )]
    pub comment: String,
}

#[axum::debug_handler]
pub async fn post_create_lyrics(
    Context { context }: Context<CreateLyricsPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditLyricsPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        album: context.params.album.clone(),
        song: context.params.song.clone(),
        kanji: context.params.kanji.clone(),
        romaji: context.params.romaji.clone(),
        english: context.params.english.clone(),
        comment: context.params.comment.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnLyrics),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_lyrics_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_lyrics_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_lyrics_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let (song_id, song_name) = validation_result.unwrap();

    if let Err(_) = validate_lyrics_dont_exist(&context.user.as_ref().unwrap().username, song_id).await {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("lyrics_exist"), String::from("User already created lyrics."))
        );
        return send_edit_lyrics_page_response(StatusCode::CONFLICT, page_context).await;
    }

    let user = context.user.unwrap();
    let username = user.username;

    let lyrics = Lyrics {
        username: username.clone(),
        song: song_id,
        kanji_content: context.params.kanji,
        romaji_content: context.params.romaji,
        english_content: context.params.english,
        ..Lyrics::default()
    };

    if let Err(error) = database::create_lyrics(lyrics).await {
        tracing::warn!("Database call failed when user {} tried to create lyrics. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_lyrics_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_lyrics_created(
            &username,
            &context.params.band,
            &context.params.album,
            &context.params.song,
            &song_name,
        ).await;
    }

    Redirect::to(
        format!("/lyrics/{}/{}/{}/", context.params.band, context.params.album, context.params.song).as_str()
    ).into_response()
}

async fn validate_lyrics_dont_exist(username: &str, song_id: i32) -> Result<bool, bool> {
    if let Ok(_) = database::get_lyrics_by_username_and_song_id(username, song_id).await {
        return Err(false);
    }
    Ok(true)
}

async fn validate_lyrics_create_form(form: &CreateLyricsPageParams) -> Result<(i32, String), Report> {
    let validation_result = validate_song_exists(&form.band, &form.album, &form.song).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("song_missing"), String::from("The specified song does not exist."))
        );
    }
    let kanji_lines = form.kanji.trim().lines().count();
    let romaji_lines = form.romaji.trim().lines().count();
    let english_lines = form.english.trim().lines().count();
    if kanji_lines != romaji_lines || kanji_lines != english_lines {
        return Err(
            create_simple_report(String::from("lines_mismatch"), String::from("Lyrics must have equal lines."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let (song_id, song_name) = validation_result.unwrap();
    Ok((song_id, song_name))
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateLyricsPageParams {
    #[route_param_source(source = "form", name = "username", default = "")]
    #[garde(skip)]
    pub username: String,

    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    #[garde(skip)]
    pub song: String,

    #[route_param_source(source = "form", name = "kanji", default = "")]
    #[garde(
        length(min = 1, max = 32000),
    )]
    pub kanji: String,

    #[route_param_source(source = "form", name = "romaji", default = "")]
    #[garde(
        length(min = 1, max = 4000),
    )]
    pub romaji: String,

    #[route_param_source(source = "form", name = "english", default = "")]
    #[garde(
        length(min = 1, max = 4000),
    )]
    pub english: String,

    #[route_param_source(source = "form", name = "comment", default = "")]
    #[garde(
        length(max = 2000),
    )]
    pub comment: String,
}

pub async fn put_update_lyrics(
    Context { context }: Context<UpdateLyricsPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditLyricsPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        album: context.params.album.clone(),
        song: context.params.song.clone(),
        kanji: context.params.kanji.clone(),
        romaji: context.params.romaji.clone(),
        english: context.params.english.clone(),
        comment: context.params.comment.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::EditLyrics)
            || user.permissions.contains(&UserPermission::EditOwnLyrics)
        },
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_lyrics_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_lyrics_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_lyrics_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let song_id = validation_result.unwrap();

    let user = &context.user.as_ref().unwrap();

    let username: &str = if user.permissions.contains(&UserPermission::EditLyrics) {
        if context.params.username.is_empty() {
            &user.username
        } else {
            context.params.username.as_str()
        }
    } else {
        &user.username
    };
    let mut existing_lyrics = database::get_lyrics_by_username_and_song_id(username, song_id).await.unwrap();
    existing_lyrics.kanji_content = context.params.kanji.clone();
    existing_lyrics.romaji_content = context.params.romaji.clone();
    existing_lyrics.english_content = context.params.english.clone();
    existing_lyrics.comment = context.params.comment.clone();

    if let Err(error) = database::update_lyrics(existing_lyrics).await {
        tracing::warn!("Database call failed when user {} tried to update lyrics. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_lyrics_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/lyrics/{}/{}/{}/", context.params.band, context.params.album, context.params.song).as_str()
    ).into_response()
}

async fn validate_lyrics_update_form(form: &UpdateLyricsPageParams) -> Result<i32, Report> {
    let validation_result = validate_song_exists(&form.band, &form.album, &form.song).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("song_missing"), String::from("The specified song does not exist."))
        );
    }
    let kanji_lines = form.kanji.trim().lines().count();
    let romaji_lines = form.romaji.trim().lines().count();
    let english_lines = form.english.trim().lines().count();
    if kanji_lines != romaji_lines || kanji_lines != english_lines {
        return Err(
            create_simple_report(String::from("lines_mismatch"), String::from("Lyrics must have equal lines."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let (song_id, _) = validation_result.unwrap();
    Ok(song_id)
}

async fn validate_song_exists(band_slug: &str, album_slug: &str, song_slug: &str) -> Result<(i32, String), Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    let album = database::get_album_by_slug_and_band_id(album_slug, band.id).await?;
    let song = database::get_song_by_slug_and_band_id(song_slug, band.id).await?;
    if !album.song_ids().contains(&song.id) {
        return Err(Box::new(
            io::Error::new(io::ErrorKind::Other, "Song doesn't exist in album.")
        ));
    }
    Ok((song.id, song.song_name.to_string()))
}

pub async fn send_edit_lyrics_page_response(status: StatusCode, context: EditLyricsPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditLyricsPageContentTemplate, &context),
                    _ => render_template!(EditLyricsPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
