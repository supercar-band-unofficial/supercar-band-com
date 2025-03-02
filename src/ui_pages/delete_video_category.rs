use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_video_category::{ DeleteVideoCategoryPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeleteVideoCategoryTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    video_category_slug: &'a str,
    video_category_title: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_video_category.html")]
pub struct DeleteVideoCategoryPageTemplate<'a> {
    active_page: &'a str,
    content: DeleteVideoCategoryTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeleteVideoCategoryPageContext>,
}
impl<'a> DeleteVideoCategoryPageTemplate<'a> {
    pub async fn new(
        context: &'a DeleteVideoCategoryPageContext
    ) -> Result<DeleteVideoCategoryPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeleteVideoCategoryPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_video_category.html", block = "page_content")]
pub struct DeleteVideoCategoryPageContentTemplate<'a> {
    content: DeleteVideoCategoryTemplateCommon<'a>,
}
impl<'a> DeleteVideoCategoryPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeleteVideoCategoryPageContext
    ) -> Result<DeleteVideoCategoryPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeleteVideoCategoryPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeleteVideoCategoryTemplateCommon<'a>) -> String {
    return format!("/editor/delete/video-category/{}/",
        content.video_category_slug,
    );
}

async fn create_common_params<'a>(context: &'a DeleteVideoCategoryPageContext) -> Result<DeleteVideoCategoryTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = true;

    let video_category_slug = &context.params.category;
    let video_category = database::get_video_category_by_slug(video_category_slug).await;

    let video_category_missing_report = Some(
        create_simple_report(String::from("video_category_missing"), String::from("Category missing."))
    );
    let mut video_category_title: String = String::from("");
    let validation_alert = get_validation_alert(
        if let Ok(video_category) = video_category {
            video_category_title = video_category.title;
            &context.params.validation_report
        } else {
            &video_category_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeleteVideoCategoryTemplateCommon {
            has_access,
            validation_alert,
            video_category_slug,
            video_category_title,
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
            if report_has_field(report, "video_category_missing") {
                message_html.push_str("<p>The specified video category does not exist.</p>");
            }
            if report_has_field(report, "videos_exist") {
                message_html.push_str("<p>The video category has videos inside of it, and cannot be deleted until all videos inside of it are deleted.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
