use std::error::Error;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };
use uuid::Uuid;

use crate::database::{ self, UserPermission, UserPreference, Photo };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_photo::{ EditPhotoPageTemplate, EditPhotoPageContentTemplate };
use crate::util::image_upload;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditPhotoPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(source = "path", name = "photo", default = "-1")]
    pub photo: i32,

    #[route_param_source(default = "")]
    pub title: String,

    #[route_param_source(default = "")]
    pub description: String,

    #[route_param_source(default = "")]
    pub temporary_photo_filename: String,
}
pub type EditPhotoPageContext = BaseContext<EditPhotoPageParams>;

pub async fn get_edit_photo(
    Context { mut context }: Context<EditPhotoPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::UploadOwnPhoto)
            || user.permissions.contains(&UserPermission::EditPhoto)
            || user.permissions.contains(&UserPermission::EditOwnPhoto)
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
                "main-article" => render_template!(EditPhotoPageContentTemplate, &context),
                _ => render_template!(EditPhotoPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreatePhotoPageParams {
    #[route_param_source(source = "form", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "form", name = "title", default = "")]
    #[garde(
        length(max = 100),
    )]
    pub title: String,

    #[route_param_source(source = "form", name = "description", default = "")]
    #[garde(
        length(max = 1000),
    )]
    pub description: String,

    
    #[route_param_source(source = "form", name = "photo", default = "")]
    #[garde(skip)]
    pub photo_upload: String,

    #[route_param_source(source = "form", name = "temporary-photo", default = "")]
    #[garde(
        custom(is_valid_image_upload(&self.temporary_photo_filename, &self.photo_upload)),
    )]
    pub temporary_photo_filename: String,
}

#[axum::debug_handler]
pub async fn post_create_photo(
    Context { context }: Context<CreatePhotoPageParams>,
) -> Response {

    let temporary_photo_filename = if context.params.photo_upload.is_empty() {
        context.params.temporary_photo_filename.clone()
    } else {
        context.params.photo_upload.clone()
    };

    let mut page_context = context.clone_with_params(EditPhotoPageParams {
        validation_report: None,
        album: context.params.album.clone(),
        photo: -1,
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        temporary_photo_filename: temporary_photo_filename.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::UploadOwnPhoto),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_photo_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_photo_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_photo_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let album_id = validation_result.unwrap();

    let mut permanent_filename = format!("{}-{}", context.params.album, Uuid::new_v4().to_string());
    match image_upload::transfer_temporary_image_upload(
        &temporary_photo_filename,
        "photos",
        &permanent_filename
    ).await {
        Ok(permanent_filename_with_extension) => {
            permanent_filename = permanent_filename_with_extension;
        },
        Err(_) => {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("image_transfer"), String::from("Image upload failed."))
            );
            return send_edit_photo_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
        }
    }

    let user = context.user.unwrap();
    let username = user.username;

    let photo = Photo {
        username: username.clone(),
        album: album_id,
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        photo_filename: permanent_filename,
        ..Photo::default()
    };

    let create_result = database::create_photo(photo).await;
    if let Err(error) = create_result {
        tracing::warn!("Database call failed when user {} tried to create photo. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_photo_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }
    let photo_id = create_result.unwrap();

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_photo_created(
            &username,
            &context.params.album,
            photo_id,
            &context.params.title,
        ).await;
    }

    Redirect::to(
        format!("/photos/{}/{}/", context.params.album, &photo_id).as_str()
    ).into_response()
}

async fn validate_photo_create_form(form: &CreatePhotoPageParams) -> Result<i32, Report> {
    let album_result = database::get_photo_album_by_slug(&form.album).await;
    if let Err(_) = album_result {
        return Err(
            create_simple_report(String::from("album_missing"), String::from("The specified album does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(album_result.unwrap().id)
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdatePhotoPageParams {
    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "path", name = "photo", default = "-1")]
    #[garde(skip)]
    pub photo: i32,

    #[route_param_source(source = "form", name = "title", default = "")]
    #[garde(
        length(max = 100),
    )]
    pub title: String,

    #[route_param_source(source = "form", name = "description", default = "")]
    #[garde(
        length(max = 1000),
    )]
    pub description: String,
}

pub async fn put_update_photo(
    Context { context }: Context<UpdatePhotoPageParams>,
) -> Response {

    let mut page_context = context.clone_with_params(EditPhotoPageParams {
        validation_report: None,
        album: context.params.album.clone(),
        photo: context.params.photo,
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        temporary_photo_filename: String::from(""),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::EditPhoto)
            || user.permissions.contains(&UserPermission::EditOwnPhoto)
        },
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_photo_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_photo_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_photo_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let mut existing_photo = validation_result.unwrap();
    let photo_id = existing_photo.id;

    let user = &context.user.as_ref().unwrap();
    let username = &user.username;

    if !user.permissions.contains(&UserPermission::EditPhoto) && *username != existing_photo.username {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_photo_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    existing_photo.title = context.params.title.clone();
    existing_photo.description = context.params.description.clone();

    if let Err(error) = database::update_photo(existing_photo).await {
        tracing::warn!("Database call failed when user {} tried to update photo. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_photo_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/photos/{}/{}/", context.params.album, photo_id).as_str()
    ).into_response()
}

async fn validate_photo_update_form(form: &UpdatePhotoPageParams) -> Result<Photo, Report> {
    let validation_result = validate_photo_exists(&form.album, form.photo).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("photo_missing"), String::from("The specified photo does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let photo = validation_result.unwrap();
    Ok(photo)
}

async fn validate_photo_exists(album_slug: &str, photo_id: i32) -> Result<Photo, Box<dyn Error>> {
    let _ = database::get_photo_album_by_slug(album_slug).await?;
    let photo = database::get_photo_by_id(photo_id).await?;
    Ok(photo)
}

pub async fn send_edit_photo_page_response(status: StatusCode, context: EditPhotoPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditPhotoPageContentTemplate, &context),
                    _ => render_template!(EditPhotoPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

fn is_valid_image_upload<'a>(new_filename: &'a str, existing_filename: &'a str) -> impl FnOnce(&str, &()) -> garde::Result + 'a {
    move |_, _| {
        if new_filename.is_empty() && existing_filename.is_empty() {
            Err(garde::Error::new("Missing file."))
        } else {
            Ok(())
        }
    }
}
