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
use crate::ui_pages::delete_photo_album::{ DeletePhotoAlbumPageTemplate, DeletePhotoAlbumPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeletePhotoAlbumPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,
}
pub type DeletePhotoAlbumPageContext = BaseContext<DeletePhotoAlbumPageParams>;

pub async fn get_delete_photo_album(
    Context { mut context }: Context<DeletePhotoAlbumPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeletePhotoAlbum)
            || user.permissions.contains(&UserPermission::DeleteOwnPhotoAlbum)
        },
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
                "main-article" => render_template!(DeletePhotoAlbumPageContentTemplate, &context),
                _ => render_template!(DeletePhotoAlbumPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmPhotoAlbumPageParams {
    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,
}

#[axum::debug_handler]
pub async fn delete_photo_album(
    Context { context }: Context<DeleteConfirmPhotoAlbumPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeletePhotoAlbumPageParams {
        validation_report: None,
        album: context.params.album.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeletePhotoAlbum)
            || user.permissions.contains(&UserPermission::DeleteOwnPhotoAlbum)
        },
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_photo_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_photo_album_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_photo_album_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let (photo_album_id, contributor) = validation_result.unwrap();
    let user = context.user.unwrap();
    let username = user.username;

    if !user.permissions.contains(&UserPermission::DeletePhotoAlbum) && username != contributor {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_photo_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    if let Err(error) = database::mark_photo_album_for_deletion(photo_album_id).await {
        tracing::warn!("Database call failed when user {} tried to delete a photo album. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_photo_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/photos/").as_str()
    ).into_response()
}

async fn validate_photo_album_delete_form(form: &DeleteConfirmPhotoAlbumPageParams) -> Result<(i32, String), Report> {
    let validation_result = validate_photo_album_exists(&form.album).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("photo_album_missing"), String::from("The specified photo album does not exist."))
        );
    }
    let (photo_album_id, photo_album_username) = validation_result.unwrap();
    if let Err(_) = database::photo_album_by_id_is_empty(photo_album_id).await {
        return Err(
            create_simple_report(String::from("photos_exist"), String::from("Deletion cannot be carried out when photos exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok((photo_album_id, photo_album_username))
}

async fn validate_photo_album_exists(album_slug: &str) -> Result<(i32, String), Box<dyn Error>> {
    let photo_album = database::get_photo_album_by_slug(album_slug).await?;
    Ok((photo_album.id, photo_album.username))
}

pub async fn send_delete_photo_album_page_response(status: StatusCode, context: DeletePhotoAlbumPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeletePhotoAlbumPageContentTemplate, &context),
                    _ => render_template!(DeletePhotoAlbumPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
