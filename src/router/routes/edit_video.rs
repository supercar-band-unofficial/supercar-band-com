use std::error::Error;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, UserPermission, UserPreference, Video };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_video::{ EditVideoPageTemplate, EditVideoPageContentTemplate };
use crate::util::format::to_kebab_case;
use crate::util::video::is_supported_video_url;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditVideoPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "category", default = "")]
    pub category: String,

    #[route_param_source(source = "path", name = "video", default = "")]
    pub video: String,

    #[route_param_source(default = "")]
    pub title: String,

    #[route_param_source(default = "")]
    pub description: String,

    #[route_param_source(default = "")]
    pub video_url: String,
}
pub type EditVideoPageContext = BaseContext<EditVideoPageParams>;

pub async fn get_edit_video(
    Context { mut context }: Context<EditVideoPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::UploadOwnVideo)
            || user.permissions.contains(&UserPermission::EditVideo)
            || user.permissions.contains(&UserPermission::EditOwnVideo)
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
                "main-article" => render_template!(EditVideoPageContentTemplate, &context),
                _ => render_template!(EditVideoPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreateVideoPageParams {
    #[route_param_source(source = "form", name = "category", default = "")]
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
    
    #[route_param_source(source = "form", name = "video-url", default = "")]
    #[garde(
        custom(is_valid_video_url(&self.video_url)),
    )]
    pub video_url: String,
}

#[axum::debug_handler]
pub async fn post_create_video(
    Context { context }: Context<CreateVideoPageParams>,
) -> Response {

    let mut page_context = context.clone_with_params(EditVideoPageParams {
        validation_report: None,
        category: context.params.category.clone(),
        video: String::from(""),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        video_url: context.params.video_url.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::UploadOwnVideo),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_video_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_video_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_video_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let category_id = validation_result.unwrap();
    let video_slug = create_video_slug(&context.params.title, category_id, -1).await;

    let user = context.user.unwrap();
    let username = user.username;

    let video = Video {
        username: username.clone(),
        category: category_id,
        slug: video_slug.clone(),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        video_url: context.params.video_url.clone(),
        ..Video::default()
    };

    let create_result = database::create_video(video).await;
    if let Err(error) = create_result {
        tracing::warn!("Database call failed when user {} tried to create video. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_video_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_video_created(
            &username,
            &context.params.category,
            &video_slug,
            &context.params.title,
        ).await;
    }

    Redirect::to(
        format!("/videos/{}/{}/", context.params.category, &video_slug).as_str()
    ).into_response()
}

async fn validate_video_create_form(form: &CreateVideoPageParams) -> Result<i32, Report> {
    let category_result = database::get_video_category_by_slug(&form.category).await;
    if let Err(_) = category_result {
        return Err(
            create_simple_report(String::from("category_missing"), String::from("The specified category does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(category_result.unwrap().id)
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateVideoPageParams {
    #[route_param_source(source = "path", name = "category", default = "")]
    #[garde(skip)]
    pub category: String,

    #[route_param_source(source = "path", name = "video", default = "")]
    #[garde(skip)]
    pub video: String,

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

    #[route_param_source(source = "form", name = "video-url", default = "")]
    #[garde(
        custom(is_valid_video_url(&self.video_url)),
    )]
    pub video_url: String,
}

pub async fn put_update_video(
    Context { context }: Context<UpdateVideoPageParams>,
) -> Response {

    let mut page_context = context.clone_with_params(EditVideoPageParams {
        validation_report: None,
        category: context.params.category.clone(),
        video: context.params.video.clone(),
        title: context.params.title.clone(),
        description: context.params.description.clone(),
        video_url: context.params.video_url.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::EditVideo)
            || user.permissions.contains(&UserPermission::EditOwnVideo)
        },
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_video_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_video_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_video_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let mut existing_video = validation_result.unwrap();
    let video_slug = create_video_slug(&context.params.title, existing_video.category, existing_video.id).await;

    let user = &context.user.as_ref().unwrap();
    let username = &user.username;

    if !user.permissions.contains(&UserPermission::EditVideo) && *username != existing_video.username {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_video_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    existing_video.slug = video_slug.clone();
    existing_video.title = context.params.title.clone();
    existing_video.description = context.params.description.clone();
    existing_video.video_url = context.params.video_url.clone();

    if let Err(error) = database::update_video(existing_video).await {
        tracing::warn!("Database call failed when user {} tried to update video. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_video_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/videos/{}/{}/", context.params.category, video_slug).as_str()
    ).into_response()
}

async fn validate_video_update_form(form: &UpdateVideoPageParams) -> Result<Video, Report> {
    let validation_result = validate_video_exists(&form.category, &form.video).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("video_missing"), String::from("The specified video does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let video = validation_result.unwrap();
    Ok(video)
}

async fn validate_video_exists(category_slug: &str, video_slug: &str) -> Result<Video, Box<dyn Error>> {
    let category = database::get_video_category_by_slug(category_slug).await?;
    let video = database::get_video_by_slug_and_category_id(video_slug, category.id).await?;
    Ok(video)
}

pub async fn send_edit_video_page_response(status: StatusCode, context: EditVideoPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditVideoPageContentTemplate, &context),
                    _ => render_template!(EditVideoPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

async fn create_video_slug(title: &str, category_id: i32, video_id: i32) -> String {
    let slug = to_kebab_case(title);
    if let Ok(video) = database::get_video_by_slug_and_category_id(&slug, category_id).await {
        if video.id != video_id {
            return format!("{}-{}", slug, video.id);
        }
    }
    return slug;
}

fn is_valid_video_url<'a>(video_url: &'a str) -> impl FnOnce(&str, &()) -> garde::Result + 'a {
    move |_, _| {
        if is_supported_video_url(video_url) {
            Ok(())
        } else {
            Err(garde::Error::new("Unsuppored url."))
        }
    }
}
