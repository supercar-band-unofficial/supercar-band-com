use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, Band, UserPermission, UserPreference };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_band::{ EditBandPageTemplate, EditBandPageContentTemplate };
use crate::util::format;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditBandPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,

    #[route_param_source(default = "")]
    pub band_name: String,
}
pub type EditBandPageContext = BaseContext<EditBandPageParams>;

pub async fn get_edit_band(
    Context { mut context }: Context<EditBandPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateBand),
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
                "main-article" => render_template!(EditBandPageContentTemplate, &context),
                _ => render_template!(EditBandPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreateBandPageParams {
    #[route_param_source(source = "form", name = "band-name", default = "")]
    #[garde(
        length(min = 1, max = 600),
    )]
    pub band_name: String,
}

#[axum::debug_handler]
pub async fn post_create_band(
    Context { context }: Context<CreateBandPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditBandPageParams {
        validation_report: None,
        band: String::from(""),
        band_name: context.params.band_name.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateBand),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_band_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_band_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_band_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let band_slug = format::to_kebab_case(&context.params.band_name);

    if let Err(_) = validate_band_dont_exist(&band_slug).await {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("band_exist"), String::from("User already created band."))
        );
        return send_edit_band_page_response(StatusCode::CONFLICT, page_context).await;
    }

    let user = context.user.unwrap();
    let username = user.username;

    let band = Band {
        band_slug: band_slug.clone(),
        band_name: context.params.band_name.clone(),
        ..Band::default()
    };

    if let Err(error) = database::create_band(band).await {
        tracing::warn!("Database call failed when user {} tried to create band. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_band_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_band_created(
            &username,
            &band_slug,
            &context.params.band_name,
        ).await;
    }

    Redirect::to(
        format!("/lyrics/{}/", band_slug).as_str()
    ).into_response()
}

async fn validate_band_dont_exist(band_slug: &str) -> Result<bool, bool> {
    if let Ok(_) = database::get_band_by_slug(band_slug).await {
        return Err(false);
    }
    Ok(true)
}

async fn validate_band_create_form(form: &CreateBandPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateBandPageParams {
    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "form", name = "band-name", default = "")]
    #[garde(
        length(min = 1, max = 600),
    )]
    pub band_name: String,
}

pub async fn put_update_band(
    Context { context }: Context<UpdateBandPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(EditBandPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        band_name: context.params.band_name.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::EditBand),
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_band_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_band_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_band_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let band_slug = format::to_kebab_case(&context.params.band_name);

    let mut existing_band = database::get_band_by_slug(&context.params.band).await.unwrap();

    if band_slug != existing_band.band_slug {
        if let Err(_) = validate_band_dont_exist(&band_slug).await {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("band_exist"), String::from("User already created band."))
            );
            return send_edit_band_page_response(StatusCode::CONFLICT, page_context).await;
        }
    }

    let user = context.user.unwrap();
    let username = user.username;

    existing_band.band_name = context.params.band_name.clone();

    if let Err(error) = database::update_band(existing_band).await {
        tracing::warn!("Database call failed when user {} tried to update band. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_band_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/lyrics/{}/", band_slug).as_str()
    ).into_response()
}

async fn validate_band_update_form(form: &UpdateBandPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

pub async fn send_edit_band_page_response(status: StatusCode, context: EditBandPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditBandPageContentTemplate, &context),
                    _ => render_template!(EditBandPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
