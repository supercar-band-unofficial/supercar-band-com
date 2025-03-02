use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, VideoCategory, UserPermission, UserPreference };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_video_category::{ EditVideoCategoryPageTemplate, EditVideoCategoryPageContentTemplate };
use crate::util::format;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditVideoCategoryPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "category", default = "")]
    pub category: String,

    #[route_param_source(default = "")]
    pub title: String,

    #[route_param_source(default = "")]
    pub description: String,
}
pub type EditVideoCategoryPageContext = BaseContext<EditVideoCategoryPageParams>;

pub async fn get_edit_video_category(
    Context { mut context }: Context<EditVideoCategoryPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnVideoCategory),
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
                "main-article" => render_template!(EditVideoCategoryPageContentTemplate, &context),
                _ => render_template!(EditVideoCategoryPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreateVideoCategoryPageParams {
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
pub async fn post_create_video_category(
    Context { context }: Context<CreateVideoCategoryPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditVideoCategoryPageParams {
        validation_report: None,
        category: String::from(""),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateOwnVideoCategory),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_video_category_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_video_category_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_video_category_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let video_category_slug = format::to_kebab_case(&context.params.title);

    if let Err(_) = validate_video_category_dont_exist(&video_category_slug).await {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("video_category_exist"), String::from("User already created video category."))
        );
        return send_edit_video_category_page_response(StatusCode::CONFLICT, page_context).await;
    }

    let user = context.user.unwrap();
    let username = user.username;

    let video_category = VideoCategory {
        username: username.clone(),
        slug: video_category_slug.clone(),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        ..VideoCategory::default()
    };

    if let Err(error) = database::create_video_category(video_category).await {
        tracing::warn!("Database call failed when user {} tried to create a video category. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_video_category_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_video_category_created(
            &username,
            &video_category_slug,
            &context.params.title,
        ).await;
    }

    Redirect::to(
        format!("/videos/{}/", video_category_slug).as_str()
    ).into_response()
}

async fn validate_video_category_dont_exist(video_category_slug: &str) -> Result<bool, bool> {
    if let Ok(_) = database::get_video_category_by_slug(video_category_slug).await {
        return Err(false);
    }
    Ok(true)
}

async fn validate_video_category_create_form(form: &CreateVideoCategoryPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateVideoCategoryPageParams {
    #[route_param_source(source = "path", name = "category", default = "")]
    #[garde(skip)]
    pub category: String,

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

pub async fn put_update_video_category(
    Context { context }: Context<UpdateVideoCategoryPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditVideoCategoryPageParams {
        validation_report: None,
        category: context.params.category.clone(),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::EditVideoCategory)
            || user.permissions.contains(&UserPermission::EditOwnVideoCategory)
        },
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_video_category_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_video_category_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_video_category_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let video_category_slug = format::to_kebab_case(&context.params.title);

    let mut existing_video_category = database::get_video_category_by_slug(&context.params.category).await.unwrap();

    if video_category_slug != existing_video_category.slug {
        if let Err(_) = validate_video_category_dont_exist(&video_category_slug).await {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("video_category_exist"), String::from("User already created video category."))
            );
            return send_edit_video_category_page_response(StatusCode::CONFLICT, page_context).await;
        }
    }

    let user = context.user.unwrap();
    let username = user.username;

    if !user.permissions.contains(&UserPermission::EditVideoCategory) && username != existing_video_category.username {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_video_category_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    existing_video_category.title = context.params.title.clone();
    existing_video_category.description = context.params.description.clone();

    if let Err(error) = database::update_video_category(existing_video_category).await {
        tracing::warn!("Database call failed when user {} tried to update a video category. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_video_category_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/videos/{}/", video_category_slug).as_str()
    ).into_response()
}

async fn validate_video_category_update_form(form: &UpdateVideoCategoryPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

pub async fn send_edit_video_category_page_response(status: StatusCode, context: EditVideoCategoryPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditVideoCategoryPageContentTemplate, &context),
                    _ => render_template!(EditVideoCategoryPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
