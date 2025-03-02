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
use crate::ui_pages::delete_album::{ DeleteAlbumPageTemplate, DeleteAlbumPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeleteAlbumPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,
}
pub type DeleteAlbumPageContext = BaseContext<DeleteAlbumPageParams>;

pub async fn get_delete_album(
    Context { mut context }: Context<DeleteAlbumPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::DeleteAlbum),
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
                "main-article" => render_template!(DeleteAlbumPageContentTemplate, &context),
                _ => render_template!(DeleteAlbumPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmAlbumPageParams {
    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,
}

#[axum::debug_handler]
pub async fn delete_album(
    Context { context }: Context<DeleteConfirmAlbumPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeleteAlbumPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        album: context.params.album.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::DeleteAlbum),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_album_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_album_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let album_id = validation_result.unwrap();
    let username = context.user.unwrap().username;

    if let Err(error) = database::mark_album_for_deletion(album_id).await {
        tracing::warn!("Database call failed when user {} tried to delete album. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/lyrics/{}/", context.params.band).as_str()
    ).into_response()
}

async fn validate_album_delete_form(form: &DeleteConfirmAlbumPageParams) -> Result<i32, Report> {
    let album_id = validate_album_exists(&form.band, &form.album).await;
    if let Err(_) = album_id {
        return Err(
            create_simple_report(String::from("album_missing"), String::from("The specified album does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(album_id.unwrap())
}

async fn validate_album_exists(band_slug: &str, album_slug: &str) -> Result<i32, Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    let album = database::get_album_by_slug_and_band_id(album_slug, band.id).await?;
    Ok(album.id)
}

pub async fn send_delete_album_page_response(status: StatusCode, context: DeleteAlbumPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeleteAlbumPageContentTemplate, &context),
                    _ => render_template!(DeleteAlbumPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
