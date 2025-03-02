use std::error::Error;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, SongTabType, SongTab, UserPermission, UserPreference };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_tabs::{ EditTabsPageTemplate, EditTabsPageContentTemplate, EditTabsSelectBandSongTemplate };
use crate::util::format;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditTabsPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    pub song: String,

    #[route_param_source(source = "path", name = "tab_type", default = "")]
    pub tab_type: String,

    #[route_param_source(source = "path", name = "contributor", default = "")]
    pub contributor: String,

    #[route_param_source(default = "")]
    pub tab_content: String,
}
pub type EditTabsPageContext = BaseContext<EditTabsPageParams>;

pub async fn get_edit_tabs(
    Context { mut context }: Context<EditTabsPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnTabs)
            || user.permissions.contains(&UserPermission::EditOwnTabs)
            || user.permissions.contains(&UserPermission::EditTabs),
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
                "main-article" => render_template!(EditTabsPageContentTemplate, &context),
                "edit-tabs-select-band-song-section" => render_template!(EditTabsSelectBandSongTemplate, &context),
                _ => render_template!(EditTabsPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreateTabsPageParams {
    #[route_param_source(source = "form", name = "band", default = "supercar")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "form", name = "song", default = "")]
    #[garde(skip)]
    pub song: String,

    #[route_param_source(source = "form", name = "tab-type", default = "")]
    #[garde(skip)]
    pub tab_type: String,

    #[route_param_source(source = "form", name = "tab-content", default = "")]
    #[garde(
        length(min = 1, max = 32000),
    )]
    pub tab_content: String,
}

#[axum::debug_handler]
pub async fn post_create_tabs(
    Context { context }: Context<CreateTabsPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditTabsPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        song: context.params.song.clone(),
        tab_type: context.params.tab_type.clone(),
        contributor: String::from(""),
        tab_content: context.params.tab_content.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnTabs),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_tabs_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_tabs_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_tabs_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let (song_id, song_name) = validation_result.unwrap();

    let tab_type = &format::to_snake_case(&context.params.tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown);

    if let Err(_) = validate_tabs_dont_exist(&context.user.as_ref().unwrap().username, &tab_type, song_id).await {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("tab_exist"), String::from("User already created tabs."))
        );
        return send_edit_tabs_page_response(StatusCode::CONFLICT, page_context).await;
    }

    let user = context.user.unwrap();
    let username = user.username;

    let tab = SongTab {
        username: username.clone(),
        song: song_id,
        tab_type: tab_type.clone(),
        tab_content: context.params.tab_content,
        ..SongTab::default()
    };

    if let Err(error) = database::create_song_tab(tab).await {
        tracing::warn!("Database call failed when user {} tried to create tabs. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_tabs_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_tabs_created(
            &username,
            &context.params.band,
            &context.params.song,
            &song_name,
            &tab_type,
        ).await;
    }

    Redirect::to(
        format!("/tabs/{}/{}/{}/{}/", context.params.band, context.params.song, context.params.tab_type, username).as_str()
    ).into_response()
}

async fn validate_tabs_dont_exist(username: &str, tab_type: &SongTabType, song_id: i32) -> Result<bool, bool> {
    if let Ok(_) = database::get_song_tab_by_username_type_and_song_id(
        username,
        tab_type,
        song_id
    ).await {
        return Err(false);
    }
    Ok(true)
}

async fn validate_tabs_create_form(form: &CreateTabsPageParams) -> Result<(i32, String), Report> {
    let validation_result = validate_song_exists(&form.band, &form.song).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("song_missing"), String::from("The specified song does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let (song_id, song_name) = validation_result.unwrap();
    Ok((song_id, song_name))
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateTabsPageParams {
    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "path", name = "song", default = "")]
    #[garde(skip)]
    pub song: String,

    #[route_param_source(source = "path", name = "tab_type", default = "")]
    #[garde(skip)]
    pub tab_type: String,

    #[route_param_source(source = "path", name = "contributor", default = "")]
    #[garde(skip)]
    pub contributor: String,

    #[route_param_source(source = "form", name = "tab-content", default = "")]
    #[garde(
        length(min = 1, max = 32000),
    )]
    pub tab_content: String,
}

pub async fn put_update_tabs(
    Context { context }: Context<UpdateTabsPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditTabsPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        song: context.params.song.clone(),
        tab_type: context.params.tab_type.clone(),
        contributor: context.params.contributor.clone(),
        tab_content: context.params.tab_content.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::EditTabs)
            || user.permissions.contains(&UserPermission::EditOwnTabs)
        },
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_tabs_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_tabs_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_tabs_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let tab_id = validation_result.unwrap();
    let tab_type = format::to_snake_case(&context.params.tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown);

    let user = &context.user.as_ref().unwrap();

    let username: &str = if user.permissions.contains(&UserPermission::EditTabs) {
        if context.params.contributor.is_empty() {
            &user.username
        } else {
            context.params.contributor.as_str()
        }
    } else {
        &user.username
    };
    let mut existing_tabs = database::get_song_tab_by_id(tab_id).await.unwrap();
    existing_tabs.tab_type = tab_type.clone();
    existing_tabs.tab_content = context.params.tab_content.clone();

    if let Err(error) = database::update_song_tab(existing_tabs).await {
        tracing::warn!("Database call failed when user {} tried to update tabs. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_tabs_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/tabs/{}/{}/{}/{}/",
            context.params.band,
            context.params.song,
            format::to_kebab_case(tab_type.to_string().as_str()),
            context.params.contributor,
        ).as_str()
    ).into_response()
}

async fn validate_tabs_update_form(form: &UpdateTabsPageParams) -> Result<i32, Report> {
    let validation_result = validate_tab_exists(&form.band, &form.song, &form.tab_type, &form.contributor).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("tab_missing"), String::from("The specified song does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let tab_id = validation_result.unwrap();
    Ok(tab_id)
}

async fn validate_song_exists(band_slug: &str, song_slug: &str) -> Result<(i32, String), Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    let song = database::get_song_by_slug_and_band_id(song_slug, band.id).await?;
    Ok((song.id, song.song_name.to_string()))
}

async fn validate_tab_exists(band_slug: &str, song_slug: &str, tab_type: &str, contributor: &str) -> Result<i32, Box<dyn Error>> {
    let song_tab_type = format::to_snake_case(tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown);
    let band = database::get_band_by_slug(band_slug).await?;
    let song = database::get_song_by_slug_and_band_id(song_slug, band.id).await?;
    let tab = database::get_song_tab_by_username_type_and_song_id(contributor, &song_tab_type, song.id).await?;
    Ok(tab.id)
}

pub async fn send_edit_tabs_page_response(status: StatusCode, context: EditTabsPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditTabsPageContentTemplate, &context),
                    _ => render_template!(EditTabsPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
