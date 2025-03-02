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
use crate::ui_pages::delete_video::{ DeleteVideoPageTemplate, DeleteVideoPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeleteVideoPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "category", default = "")]
    pub category: String,

    #[route_param_source(source = "path", name = "video", default = "")]
    pub video: String,
}
pub type DeleteVideoPageContext = BaseContext<DeleteVideoPageParams>;

pub async fn get_delete_video(
    Context { mut context }: Context<DeleteVideoPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeleteVideo)
            || user.permissions.contains(&UserPermission::DeleteOwnVideo)
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
                "main-article" => render_template!(DeleteVideoPageContentTemplate, &context),
                _ => render_template!(DeleteVideoPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmVideoPageParams {
    #[route_param_source(source = "path", name = "category", default = "")]
    #[garde(skip)]
    pub category: String,

    #[route_param_source(source = "path", name = "video", default = "")]
    #[garde(skip)]
    pub video: String,
}

#[axum::debug_handler]
pub async fn delete_video(
    Context { context }: Context<DeleteConfirmVideoPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeleteVideoPageParams {
        validation_report: None,
        category: context.params.category.clone(),
        video: context.params.video.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeleteVideo)
            || user.permissions.contains(&UserPermission::DeleteOwnVideo)
        },
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_video_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_video_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_video_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let (video_id, contributor) = validation_result.unwrap();
    let user = context.user.unwrap();
    let username = user.username;

    if !user.permissions.contains(&UserPermission::DeleteVideo) && username != contributor {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_video_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    if let Err(error) = database::mark_video_for_deletion(video_id).await {
        tracing::warn!("Database call failed when user {} tried to delete a video. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_video_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/videos/{}/", &context.params.category).as_str()
    ).into_response()
}

async fn validate_video_delete_form(form: &DeleteConfirmVideoPageParams) -> Result<(i32, String), Report> {
    let validation_result = validate_video_exists(&form.category, &form.video).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("video_missing"), String::from("The specified video does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(validation_result.unwrap())
}

async fn validate_video_exists(category_slug: &str, video_slug: &str) -> Result<(i32, String), Box<dyn Error>> {
    let category = database::get_video_category_by_slug(category_slug).await?;
    let video = database::get_video_by_slug_and_category_id(video_slug, category.id).await?;
    Ok((video.id, video.username))
}

pub async fn send_delete_video_page_response(status: StatusCode, context: DeleteVideoPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeleteVideoPageContentTemplate, &context),
                    _ => render_template!(DeleteVideoPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
