
use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, UserPermission, User };
use crate::router::authn::verify_password;
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_profile_password::{ EditProfilePasswordPageTemplate, EditProfilePasswordPageContentTemplate };
use crate::util::user::create_user_profile_href;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditProfilePasswordPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(default = "")]
    pub new_password: String,
}
pub type EditProfilePasswordPageContext = BaseContext<EditProfilePasswordPageParams>;

pub async fn get_edit_profile_password(
    Context { mut context }: Context<EditProfilePasswordPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::EditOwnProfileInfo),
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
                "main-article" => render_template!(EditProfilePasswordPageContentTemplate, &context),
                _ => render_template!(EditProfilePasswordPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateProfilePasswordPageParams {
    #[route_param_source(source = "form", name = "current-password", default = "")]
    #[garde(
        length(min = 1),
    )]
    pub current_password: String,

    #[route_param_source(source = "form", name = "new-password", default = "")]
    #[garde(
        length(min = 1),
        // All visible characters allowed
        pattern(r"^[\x20-\x7E\xA0-\u10FFFF]*$")
    )]
    pub new_password: String,
}

pub async fn put_update_profile_password(
    Context { context }: Context<UpdateProfilePasswordPageParams>,
) -> Response {

    let mut page_context = context.clone_with_params(EditProfilePasswordPageParams {
        validation_report: None,
        new_password: context.params.new_password.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::EditOwnProfileInfo),
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_profile_password_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let user = context.user.as_ref().unwrap();
    let username = user.username.clone();

    let validation_result = validate_profile_password_update_form(&context.params, &username).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_profile_password_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    if let Err(error) = database::update_user_password(&username, &context.params.new_password).await {
        tracing::warn!("Database call failed when user {} tried to update their password. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_profile_password_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        create_user_profile_href(&username).as_str()
    ).into_response()
}

async fn validate_profile_password_update_form(form: &UpdateProfilePasswordPageParams, username: &str) -> Result<User, Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let existing_user = database::get_user_by_username(username).await;
    if let Err(_) = existing_user {
        return Err(
            create_simple_report(String::from("server_error"), String::from("User not found."))
        );
    }
    if let Err(_) = verify_password(existing_user.as_ref().unwrap().password.as_str(), form.current_password.as_str()) {
        return Err(
            create_simple_report(String::from("current_password"), String::from("Bad password entry."))
        );
    }
    Ok(existing_user.unwrap())
}

pub async fn send_edit_profile_password_page_response(status: StatusCode, context: EditProfilePasswordPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditProfilePasswordPageContentTemplate, &context),
                    _ => render_template!(EditProfilePasswordPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}
