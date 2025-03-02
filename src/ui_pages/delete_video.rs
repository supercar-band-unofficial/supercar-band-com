use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_video::{ DeleteVideoPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeleteVideoTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    video_category_slug: &'a str,
    video_slug: &'a str,
    video_title: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_video.html")]
pub struct DeleteVideoPageTemplate<'a> {
    active_page: &'a str,
    content: DeleteVideoTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeleteVideoPageContext>,
}
impl<'a> DeleteVideoPageTemplate<'a> {
    pub async fn new(
        context: &'a DeleteVideoPageContext
    ) -> Result<DeleteVideoPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeleteVideoPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_video.html", block = "page_content")]
pub struct DeleteVideoPageContentTemplate<'a> {
    content: DeleteVideoTemplateCommon<'a>,
}
impl<'a> DeleteVideoPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeleteVideoPageContext
    ) -> Result<DeleteVideoPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeleteVideoPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeleteVideoTemplateCommon<'a>) -> String {
    return format!("/editor/delete/video/{}/{}/",
        content.video_category_slug, content.video_slug,
    );
}

async fn create_common_params<'a>(context: &'a DeleteVideoPageContext) -> Result<DeleteVideoTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = true;

    let video_category_slug = &context.params.category;
    let video_slug = &context.params.video;
    let category = database::get_video_category_by_slug(video_category_slug).await?;
    let video = database::get_video_by_slug_and_category_id(video_slug, category.id).await;

    let video_missing_report = Some(
        create_simple_report(String::from("video_missing"), String::from("Video missing."))
    );
    let mut video_title: String = String::from("");
    let validation_alert = get_validation_alert(
        if let Ok(video) = video {
            video_title = video.title;
            &context.params.validation_report
        } else {
            &video_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeleteVideoTemplateCommon {
            has_access,
            validation_alert,
            video_category_slug,
            video_slug,
            video_title,
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
            if report_has_field(report, "video_missing") {
                message_html.push_str("<p>The specified video does not exist.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
