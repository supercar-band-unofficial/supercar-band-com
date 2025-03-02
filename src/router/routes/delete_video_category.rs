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
use crate::ui_pages::delete_video_category::{ DeleteVideoCategoryPageTemplate, DeleteVideoCategoryPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeleteVideoCategoryPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "category", default = "")]
    pub category: String,
}
pub type DeleteVideoCategoryPageContext = BaseContext<DeleteVideoCategoryPageParams>;

pub async fn get_delete_video_category(
    Context { mut context }: Context<DeleteVideoCategoryPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeleteVideoCategory)
            || user.permissions.contains(&UserPermission::DeleteOwnVideoCategory)
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
                "main-article" => render_template!(DeleteVideoCategoryPageContentTemplate, &context),
                _ => render_template!(DeleteVideoCategoryPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmVideoCategoryPageParams {
    #[route_param_source(source = "path", name = "category", default = "")]
    #[garde(skip)]
    pub category: String,
}

#[axum::debug_handler]
pub async fn delete_video_category(
    Context { context }: Context<DeleteConfirmVideoCategoryPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeleteVideoCategoryPageParams {
        validation_report: None,
        category: context.params.category.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::DeleteVideoCategory)
            || user.permissions.contains(&UserPermission::DeleteOwnVideoCategory)
        },
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_video_category_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_video_category_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_video_category_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let (video_category_id, contributor) = validation_result.unwrap();
    let user = context.user.unwrap();
    let username = user.username;

    if !user.permissions.contains(&UserPermission::DeleteVideoCategory) && username != contributor {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_video_category_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    if let Err(error) = database::mark_video_category_for_deletion(video_category_id).await {
        tracing::warn!("Database call failed when user {} tried to delete a video category. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_video_category_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/videos/").as_str()
    ).into_response()
}

async fn validate_video_category_delete_form(form: &DeleteConfirmVideoCategoryPageParams) -> Result<(i32, String), Report> {
    let validation_result = validate_video_category_exists(&form.category).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("video_category_missing"), String::from("The specified video category does not exist."))
        );
    }
    let (video_category_id, video_category_username) = validation_result.unwrap();
    if let Err(_) = database::video_category_by_id_is_empty(video_category_id).await {
        return Err(
            create_simple_report(String::from("videos_exist"), String::from("Deletion cannot be carried out when videos exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok((video_category_id, video_category_username))
}

async fn validate_video_category_exists(category_slug: &str) -> Result<(i32, String), Box<dyn Error>> {
    let video_category = database::get_video_category_by_slug(category_slug).await?;
    Ok((video_category.id, video_category.username))
}

pub async fn send_delete_video_category_page_response(status: StatusCode, context: DeleteVideoCategoryPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeleteVideoCategoryPageContentTemplate, &context),
                    _ => render_template!(DeleteVideoCategoryPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
