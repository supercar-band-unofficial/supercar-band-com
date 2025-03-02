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
use crate::ui_pages::delete_band::{ DeleteBandPageTemplate, DeleteBandPageContentTemplate };
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct DeleteBandPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,
}
pub type DeleteBandPageContext = BaseContext<DeleteBandPageParams>;

pub async fn get_delete_band(
    Context { mut context }: Context<DeleteBandPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::DeleteBand),
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
                "main-article" => render_template!(DeleteBandPageContentTemplate, &context),
                _ => render_template!(DeleteBandPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct DeleteConfirmBandPageParams {
    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,
}

#[axum::debug_handler]
pub async fn delete_band(
    Context { context }: Context<DeleteConfirmBandPageParams>,
) -> Response {
    let mut page_context = context.clone_with_params(DeleteBandPageParams {
        validation_report: None,
        band: context.params.band.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::DeleteBand),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_delete_band_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_band_delete_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_delete_band_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let band_id = validation_result.unwrap();
    let username = context.user.unwrap().username;

    if let Err(error) = database::mark_band_for_deletion(band_id).await {
        tracing::warn!("Database call failed when user {} tried to delete band. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_delete_band_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    Redirect::to(
        format!("/lyrics/").as_str()
    ).into_response()
}

async fn validate_band_delete_form(form: &DeleteConfirmBandPageParams) -> Result<i32, Report> {
    let band_id = validate_band_exists(&form.band).await;
    if let Err(_) = band_id {
        return Err(
            create_simple_report(String::from("band_missing"), String::from("The specified band does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(band_id.unwrap())
}

async fn validate_band_exists(band_slug: &str) -> Result<i32, Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    Ok(band.id)
}

pub async fn send_delete_band_page_response(status: StatusCode, context: DeleteBandPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(DeleteBandPageContentTemplate, &context),
                    _ => render_template!(DeleteBandPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
