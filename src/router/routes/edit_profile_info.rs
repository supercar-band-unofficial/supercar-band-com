
use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, UserPermission, User, UserGender };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_profile_info::{ EditProfileInfoPageTemplate, EditProfileInfoPageContentTemplate };
use crate::util::user::create_user_profile_href;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditProfileInfoPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(default = "")]
    pub first_name: String,

    #[route_param_source(default = "")]
    pub last_name: String,

    #[route_param_source(default = "")]
    pub email: String,

    #[route_param_source(default = "")]
    pub gender: String,

    #[route_param_source(default = "")]
    pub country: String,

    #[route_param_source(default = "")]
    pub about_me: String,
}
pub type EditProfileInfoPageContext = BaseContext<EditProfileInfoPageParams>;

pub async fn get_edit_profile_info(
    Context { mut context }: Context<EditProfileInfoPageParams>,
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
                "main-article" => render_template!(EditProfileInfoPageContentTemplate, &context),
                _ => render_template!(EditProfileInfoPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateProfileInfoPageParams {
    #[route_param_source(source = "form", name = "first-name", default = "")]
    #[garde(
        length(min = 1, max = 30),
    )]
    pub first_name: String,

    #[route_param_source(source = "form", name = "last-name", default = "")]
    #[garde(
        length(max = 30),
    )]
    pub last_name: String,

    #[route_param_source(source = "form", name = "email", default = "")]
    #[garde(
        length(max = 320),
    )]
    pub email: String,

    #[route_param_source(source = "form", name = "gender", default = "")]
    #[garde(
        custom(is_valid_gender(&self.gender)),
    )]
    pub gender: String,

    #[route_param_source(source = "form", name = "country", default = "")]
    #[garde(
        length(max = 1024),
    )]
    pub country: String,

    #[route_param_source(source = "form", name = "about-me", default = "")]
    #[garde(
        length(max = 4096),
    )]
    pub about_me: String,
}

pub async fn put_update_profile_info(
    Context { context }: Context<UpdateProfileInfoPageParams>,
) -> Response {

    let mut page_context = context.clone_with_params(EditProfileInfoPageParams {
        validation_report: None,
        first_name: context.params.first_name.clone(),
        last_name: context.params.last_name.clone(),
        email: context.params.email.clone(),
        gender: context.params.gender.clone(),
        country: context.params.country.clone(),
        about_me: context.params.about_me.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::EditOwnProfileInfo),
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_profile_info_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let user = context.user.as_ref().unwrap();
    let username = user.username.clone();

    let validation_result = validate_profile_info_update_form(&context.params, &username).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_profile_info_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    
    let mut existing_user = validation_result.unwrap();

    existing_user.first_name = context.params.first_name.clone();
    existing_user.last_name = context.params.last_name.clone();
    existing_user.email = context.params.email.clone();
    existing_user.gender = UserGender::from(context.params.gender.as_str());
    existing_user.country = context.params.country.clone();
    existing_user.about_me = context.params.about_me.clone();

    if let Err(error) = database::update_user_profile_info(existing_user).await {
        tracing::warn!("Database call failed when user {} tried to update user profile info. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_profile_info_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        create_user_profile_href(&username).as_str()
    ).into_response()
}

async fn validate_profile_info_update_form(form: &UpdateProfileInfoPageParams, username: &str) -> Result<User, Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let existing_user = database::get_user_by_username(username).await;
    if let Err(_) = existing_user {
        return Err(
            create_simple_report(String::from("server_error"), String::from("User not found."))
        );
    }
    Ok(existing_user.unwrap())
}

pub async fn send_edit_profile_info_page_response(status: StatusCode, context: EditProfileInfoPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditProfileInfoPageContentTemplate, &context),
                    _ => render_template!(EditProfileInfoPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

fn is_valid_gender<'a>(gender: &'a str) -> impl FnOnce(&str, &()) -> garde::Result + 'a {
    move |_, _| {
        if gender == "Male" || gender == "Female" || gender == "Unknown" {
            Ok(())
        } else {
            Err(garde::Error::new("Unsuppored url."))
        }
    }
}
