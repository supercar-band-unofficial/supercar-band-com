use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::edit_video_category::{ EditVideoCategoryPageContext };
use crate::router::validation::report_has_field;

struct EditVideoCategoryTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    selected_category_slug: String,
    title: String,
    description: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_video_category.html")]
pub struct EditVideoCategoryPageTemplate<'a> {
    active_page: &'a str,
    content: EditVideoCategoryTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditVideoCategoryPageContext>,
}
impl<'a> EditVideoCategoryPageTemplate<'a> {
    pub async fn new(
        context: &'a EditVideoCategoryPageContext
    ) -> Result<EditVideoCategoryPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditVideoCategoryPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_video_category.html", block = "page_content")]
pub struct EditVideoCategoryPageContentTemplate<'a> {
    content: EditVideoCategoryTemplateCommon<'a>,
}
impl<'a> EditVideoCategoryPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditVideoCategoryPageContext
    ) -> Result<EditVideoCategoryPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditVideoCategoryPageContentTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditVideoCategoryTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Create Video Category",
        _ => "Edit Video Category",
    }
}

fn get_cancel_href<'a>(content: &EditVideoCategoryTemplateCommon<'a>) -> String {
    if content.is_create {
        return format!("/videos/");
    }
    return format!("/videos/{}/",
        content.selected_category_slug,
    );
}

fn get_submit_action<'a>(content: &EditVideoCategoryTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/video-category/");
    }
    return format!("/editor/update/video-category/{}/",
        content.selected_category_slug
    );
}

async fn create_common_params<'a>(context: &'a EditVideoCategoryPageContext) -> Result<EditVideoCategoryTemplateCommon<'a>, Box<dyn Error>> {

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_category_slug = &context.params.category;

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (title, description) = if is_create || validation_alert.is_some() {
        (
            context.params.title.clone(),
            context.params.description.clone(),
        )
    } else {
        let category = database::get_video_category_by_slug(selected_category_slug).await?;
        (
            category.title,
            category.description,
        )
    };

    Ok(
        EditVideoCategoryTemplateCommon {
            is_create,
            has_access,
            validation_alert,
            title,
            description,
            selected_category_slug: selected_category_slug.to_string(),
        }
    )
}

fn get_validation_alert<'a>(report: &Option<Report>) -> Option<AlertTemplate<'a>> {
    match report {
        Some(report) => {
            let mut message_html: String = "".to_owned();

            if report_has_field(report, "server_error") {
                message_html.push_str("<p>A system error occurred. Please try again later.</p>");
            }
            if report_has_field(report, "forbidden") {
                message_html.push_str("<p>You do not have sufficient permissions to use this form.</p>");
            }
            if report_has_field(report, "video_category_exist") {
                message_html.push_str("<p>A video category with this name already exists. Please edit the existing video category.</p>");
            }
            if report_has_field(report, "video_category_missing") {
                message_html.push_str("<p>The video category doesn't exist. It may have been deleted after visiting this page.</p>");
            }
            if report_has_field(report, "title") {
                message_html.push_str("<p><strong>Title:</strong> This field is required.</p>");
            }
            if report_has_field(report, "description") {
                message_html.push_str("<p><strong>Description:</strong> Invalid input.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
