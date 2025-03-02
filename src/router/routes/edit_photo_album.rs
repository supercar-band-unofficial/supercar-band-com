use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, PhotoAlbum, UserPermission, UserPreference };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_photo_album::{ EditPhotoAlbumPageTemplate, EditPhotoAlbumPageContentTemplate };
use crate::util::format;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditPhotoAlbumPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(default = "")]
    pub title: String,

    #[route_param_source(default = "")]
    pub description: String,
}
pub type EditPhotoAlbumPageContext = BaseContext<EditPhotoAlbumPageParams>;

pub async fn get_edit_photo_album(
    Context { mut context }: Context<EditPhotoAlbumPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnPhotoAlbum),
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
                "main-article" => render_template!(EditPhotoAlbumPageContentTemplate, &context),
                _ => render_template!(EditPhotoAlbumPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreatePhotoAlbumPageParams {
    #[route_param_source(source = "form", name = "title", default = "")]
    #[garde(
        length(min = 1, max = 100),
    )]
    pub title: String,

    #[route_param_source(source = "form", name = "description", default = "")]
    #[garde(
        length(max = 1000),
    )]
    pub description: String,
}

#[axum::debug_handler]
pub async fn post_create_photo_album(
    Context { context }: Context<CreatePhotoAlbumPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditPhotoAlbumPageParams {
        validation_report: None,
        album: String::from(""),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnPhotoAlbum),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_photo_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_photo_album_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_photo_album_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let photo_album_slug = format::to_kebab_case(&context.params.title);

    if let Err(_) = validate_photo_album_dont_exist(&photo_album_slug).await {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("photo_album_exist"), String::from("User already created photo_album."))
        );
        return send_edit_photo_album_page_response(StatusCode::CONFLICT, page_context).await;
    }

    let user = context.user.unwrap();
    let username = user.username;

    let photo_album = PhotoAlbum {
        username: username.clone(),
        slug: photo_album_slug.clone(),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        ..PhotoAlbum::default()
    };

    if let Err(error) = database::create_photo_album(photo_album).await {
        tracing::warn!("Database call failed when user {} tried to create a photo album. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_photo_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_photo_album_created(
            &username,
            &photo_album_slug,
            &context.params.title,
        ).await;
    }

    Redirect::to(
        format!("/photos/{}/", photo_album_slug).as_str()
    ).into_response()
}

async fn validate_photo_album_dont_exist(photo_album_slug: &str) -> Result<bool, bool> {
    if let Ok(_) = database::get_photo_album_by_slug(photo_album_slug).await {
        return Err(false);
    }
    Ok(true)
}

async fn validate_photo_album_create_form(form: &CreatePhotoAlbumPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdatePhotoAlbumPageParams {
    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "form", name = "title", default = "")]
    #[garde(
        length(min = 1, max = 100),
    )]
    pub title: String,

    #[route_param_source(source = "form", name = "description", default = "")]
    #[garde(
        length(max = 1000),
    )]
    pub description: String,
}

pub async fn put_update_photo_album(
    Context { context }: Context<UpdatePhotoAlbumPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditPhotoAlbumPageParams {
        validation_report: None,
        album: context.params.album.clone(),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::EditPhotoAlbum)
            || user.permissions.contains(&UserPermission::EditOwnPhotoAlbum)
        },
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_photo_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_photo_album_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_photo_album_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let photo_album_slug = format::to_kebab_case(&context.params.title);

    let mut existing_photo_album = database::get_photo_album_by_slug(&context.params.album).await.unwrap();

    if photo_album_slug != existing_photo_album.slug {
        if let Err(_) = validate_photo_album_dont_exist(&photo_album_slug).await {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("photo_album_exist"), String::from("User already created photo album."))
            );
            return send_edit_photo_album_page_response(StatusCode::CONFLICT, page_context).await;
        }
    }

    let user = context.user.unwrap();
    let username = user.username;

    if !user.permissions.contains(&UserPermission::EditPhotoAlbum) && username != existing_photo_album.username {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_photo_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    existing_photo_album.title = context.params.title.clone();
    existing_photo_album.description = context.params.description.clone();

    if let Err(error) = database::update_photo_album(existing_photo_album).await {
        tracing::warn!("Database call failed when user {} tried to update a photo album. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_photo_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/photos/{}/", photo_album_slug).as_str()
    ).into_response()
}

async fn validate_photo_album_update_form(form: &UpdatePhotoAlbumPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

pub async fn send_edit_photo_album_page_response(status: StatusCode, context: EditPhotoAlbumPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditPhotoAlbumPageContentTemplate, &context),
                    _ => render_template!(EditPhotoAlbumPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
