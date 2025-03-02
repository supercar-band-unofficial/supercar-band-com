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
use crate::ui_pages::delete_photo::{ DeletePhotoPageTemplate, DeletePhotoPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeletePhotoPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "path", name = "photo", default = "-1")]
    pub photo: i32,
}
pub type DeletePhotoPageContext = BaseContext<DeletePhotoPageParams>;

pub async fn get_delete_photo(
    Context { mut context }: Context<DeletePhotoPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeletePhoto)
            || user.permissions.contains(&UserPermission::DeleteOwnPhoto)
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
                "main-article" => render_template!(DeletePhotoPageContentTemplate, &context),
                _ => render_template!(DeletePhotoPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmPhotoPageParams {
    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "path", name = "photo", default = "-1")]
    #[garde(skip)]
    pub photo: i32,
}

#[axum::debug_handler]
pub async fn delete_photo(
    Context { context }: Context<DeleteConfirmPhotoPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeletePhotoPageParams {
        validation_report: None,
        album: context.params.album.clone(),
        photo: context.params.photo,
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeletePhoto)
            || user.permissions.contains(&UserPermission::DeleteOwnPhoto)
        },
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_photo_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_photo_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_photo_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let (photo_id, contributor) = validation_result.unwrap();
    let user = context.user.unwrap();
    let username = user.username;

    if !user.permissions.contains(&UserPermission::DeletePhoto) && username != contributor {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_photo_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    if let Err(error) = database::mark_photo_for_deletion(photo_id).await {
        tracing::warn!("Database call failed when user {} tried to delete a photo. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_photo_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/photos/{}/", &context.params.album).as_str()
    ).into_response()
}

async fn validate_photo_delete_form(form: &DeleteConfirmPhotoPageParams) -> Result<(i32, String), Report> {
    let validation_result = validate_photo_exists(&form.album, form.photo).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("photo_missing"), String::from("The specified photo does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(validation_result.unwrap())
}

async fn validate_photo_exists(album_slug: &str, photo_id: i32) -> Result<(i32, String), Box<dyn Error>> {
    let _ = database::get_photo_album_by_slug(album_slug).await?;
    let photo = database::get_photo_by_id(photo_id).await?;
    Ok((photo.id, photo.username))
}

pub async fn send_delete_photo_page_response(status: StatusCode, context: DeletePhotoPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeletePhotoPageContentTemplate, &context),
                    _ => render_template!(DeletePhotoPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
