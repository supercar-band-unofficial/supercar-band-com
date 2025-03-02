use std::error::Error;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, SongTabType, UserPermission };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::delete_tabs::{ DeleteTabsPageTemplate, DeleteTabsPageContentTemplate };
use crate::util::format;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeleteTabsPageParams {
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
}
pub type DeleteTabsPageContext = BaseContext<DeleteTabsPageParams>;

pub async fn get_delete_tabs(
    Context { mut context }: Context<DeleteTabsPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::DeleteOwnTabs)
            || user.permissions.contains(&UserPermission::DeleteTabs),
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
                "main-article" => render_template!(DeleteTabsPageContentTemplate, &context),
                _ => render_template!(DeleteTabsPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmTabsPageParams {
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
}

#[axum::debug_handler]
pub async fn delete_tabs(
    Context { context }: Context<DeleteConfirmTabsPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeleteTabsPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        song: context.params.song.clone(),
        tab_type: context.params.tab_type.clone(),
        contributor: context.params.contributor.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeleteTabs) || (
                user.permissions.contains(&UserPermission::DeleteOwnTabs)
                && user.username == context.params.contributor
            )
        },
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_tabs_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_tabs_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_tabs_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let tab_id = validation_result.unwrap();
    let username = context.user.unwrap().username;

    if let Err(error) = database::mark_tab_for_deletion(tab_id).await {
        tracing::warn!("Database call failed when user {} tried to delete tabs. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_tabs_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/tabs/{}/{}/", context.params.band, context.params.song).as_str()
    ).into_response()
}

async fn validate_tabs_delete_form(form: &DeleteConfirmTabsPageParams) -> Result<i32, Report> {
    let tab_id = validate_tabs_exists(&form.band, &form.song, &form.tab_type, &form.contributor).await;
    if let Err(_) = tab_id {
        return Err(
            create_simple_report(String::from("tab_missing"), String::from("The specified tab does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(tab_id.unwrap())
}

async fn validate_tabs_exists(band_slug: &str, song_slug: &str, tab_type: &str, contributor: &str) -> Result<i32, Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    let song = database::get_song_by_slug_and_band_id(song_slug, band.id).await?;
    let song_tab_type = &format::to_snake_case(tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown);
    let tab = database::get_song_tab_by_username_type_and_song_id(contributor, &song_tab_type, song.id).await?;
    Ok(tab.id)
}

pub async fn send_delete_tabs_page_response(status: StatusCode, context: DeleteTabsPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeleteTabsPageContentTemplate, &context),
                    _ => render_template!(DeleteTabsPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
