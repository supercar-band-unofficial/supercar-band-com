
use axum::{
    http::{ StatusCode },
    http::header::HeaderValue,
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, User, UserPermission };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_profile_picture::{ EditProfilePicturePageTemplate, EditProfilePicturePageContentTemplate };
use crate::util::image_upload;
use crate::util::user::create_user_profile_href;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditProfilePicturePageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(default = "")]
    pub temporary_profile_picture_filename: String,
}
pub type EditProfilePicturePageContext = BaseContext<EditProfilePicturePageParams>;

pub async fn get_edit_profile_picture(
    Context { mut context }: Context<EditProfilePicturePageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::UploadOwnProfilePicture),
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
                "main-article" => render_template!(EditProfilePicturePageContentTemplate, &context),
                _ => render_template!(EditProfilePicturePageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateProfilePicturePageParams {
    #[route_param_source(source = "form", name = "profile-picture", default = "")]
    #[garde(skip)]
    pub profile_picture_upload: String,

    #[route_param_source(source = "form", name = "temporary-profile-picture", default = "")]
    #[garde(
        custom(is_valid_image_upload(&self.temporary_profile_picture_filename, &self.profile_picture_upload)),
    )]
    pub temporary_profile_picture_filename: String,
}

pub async fn put_update_profile_picture(
    Context { context }: Context<UpdateProfilePicturePageParams>,
) -> Response {

    let temporary_profile_picture_filename = if !context.params.temporary_profile_picture_filename.is_empty() {
        context.params.temporary_profile_picture_filename.clone()
    } else {
        context.params.profile_picture_upload.clone()
    };

    let mut page_context = context.clone_with_params(EditProfilePicturePageParams {
        validation_report: None,
        temporary_profile_picture_filename: temporary_profile_picture_filename.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::UploadOwnProfilePicture),
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_profile_picture_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let user = &context.user.as_ref().unwrap();
    let username = &user.username;

    let validation_result = validate_profile_picture_update_form(&context.params, &username).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_profile_picture_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let mut permanent_filename = format!("{}", username);
    if !temporary_profile_picture_filename.is_empty() {
        match image_upload::transfer_temporary_image_upload(
            &temporary_profile_picture_filename,
            "profile-pictures",
            &permanent_filename
        ).await {
            Ok(permanent_filename_with_extension) => {
                permanent_filename = permanent_filename_with_extension;
            },
            Err(_) => {
                page_context.params.validation_report = Some(
                    create_simple_report(String::from("image_transfer"), String::from("Image upload failed."))
                );
                return send_edit_profile_picture_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
            }
        }
    }

    if let Err(error) = database::update_user_profile_picture(username, &permanent_filename).await {
        tracing::warn!("Database call failed when user {} tried to update their profile picture. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_profile_picture_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    let mut response = Redirect::to(
        format!("{}?hx-refresh=true", create_user_profile_href(username)).as_str()
    ).into_response();

    response.headers_mut().insert(
        "HX-Refresh",
        HeaderValue::from_static("true"),
    );

    response
}

async fn validate_profile_picture_update_form(form: &UpdateProfilePicturePageParams, username: &str) -> Result<User, Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let validation_result = database::get_user_by_username(username).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("system_error"), String::from("User does not exist."))
        );
    }
    let user = validation_result.unwrap();
    Ok(user)
}

pub async fn send_edit_profile_picture_page_response(status: StatusCode, context: EditProfilePicturePageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditProfilePicturePageContentTemplate, &context),
                    _ => render_template!(EditProfilePicturePageTemplate, &context),
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
