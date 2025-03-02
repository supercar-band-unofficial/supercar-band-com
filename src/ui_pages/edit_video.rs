use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self, VideoCategoryWithPreview };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::edit_video::{ EditVideoPageContext };
use crate::router::validation::report_has_field;

struct EditVideoTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    categories: Vec<VideoCategoryWithPreview>,
    selected_category_slug: String,
    selected_video_slug: String,
    validation_alert: Option<AlertTemplate<'a>>,
    link_info: AlertTemplate<'a>,
    title: String,
    description: String,
    video_url: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_video.html")]
pub struct EditVideoPageTemplate<'a> {
    active_page: &'a str,
    content: EditVideoTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditVideoPageContext>,
}
impl<'a> EditVideoPageTemplate<'a> {
    pub async fn new(
        context: &'a EditVideoPageContext
    ) -> Result<EditVideoPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditVideoPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_video.html", block = "page_content")]
pub struct EditVideoPageContentTemplate<'a> {
    content: EditVideoTemplateCommon<'a>,
}
impl<'a> EditVideoPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditVideoPageContext
    ) -> Result<EditVideoPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditVideoPageContentTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditVideoTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Add Video",
        _ => "Edit Video",
    }
}

fn get_submit_action<'a>(content: &EditVideoTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/video/");
    }
    return format!("/editor/update/video/{}/{}/",
        content.selected_category_slug, content.selected_video_slug,
    );
}

fn get_cancel_href<'a>(content: &EditVideoTemplateCommon<'a>) -> String {
    if content.is_create {
        return format!("/videos/{}/", content.selected_category_slug);
    }
    return format!("/videos/{}/{}/",
        content.selected_category_slug, content.selected_video_slug,
    );
}

async fn create_common_params<'a>(context: &'a EditVideoPageContext) -> Result<EditVideoTemplateCommon<'a>, Box<dyn Error>> {
    let categories = database::get_all_video_categories().await?;

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_category_slug = &context.params.category;
    let selected_video_slug = &context.params.video;

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (
        title, description, video_url,
    ) = if is_create || validation_alert.is_some() {
        (
            context.params.title.clone(),
            context.params.description.clone(),
            context.params.video_url.clone(),
        )
    } else {
        let video_category = database::get_video_category_by_slug(selected_category_slug).await?;
        let video = database::get_video_by_slug_and_category_id(selected_video_slug, video_category.id).await?;
        (
            video.title,
            video.description,
            video.video_url,
        )
    };

    let link_info = AlertTemplate {
        variant: "info",
        message_html: String::from(r#"
            <p>You may provide video links from the following websites:<br>
                <strong>youtube.com</strong>,
                <strong>dailymotion.com</strong>,
                <strong>nicovideo.jp</strong>
            </p>
        "#),
    };

    Ok(
        EditVideoTemplateCommon {
            is_create,
            has_access,

            categories,
            selected_category_slug: selected_category_slug.to_string(),
            selected_video_slug: selected_video_slug.to_string(),
            validation_alert,
            link_info,
            title,
            description,
            video_url,
        }
    )
}

fn get_validation_alert<'a>(report: &Option<Report>) -> Option<AlertTemplate<'a>> {
    match report {
        Some(report) => {
            let mut message_html: String = "".to_owned();

            if report_has_field(report, "server_error") {
                message_html.push_str("<p>A system error occurred. Please notify the site admins if this continues to happen.</p>");
            }
            if report_has_field(report, "forbidden") {
                message_html.push_str("<p>You do not have sufficient permissions to use this form.</p>");
            }
            if report_has_field(report, "video_exist") {
                message_html.push_str("<p>Another video with the same name already exists.</p>");
            }
            if report_has_field(report, "category_missing") {
                message_html.push_str("<p>The video category you are adding to does not exist. Maybe it was deleted since loading this page.</p>");
            }
            if report_has_field(report, "title") {
                message_html.push_str("<p><strong>Title:</strong> Invalid entry.</p>");
            }
            if report_has_field(report, "description") {
                message_html.push_str("<p><strong>Description:</strong> Invalid entry.</p>");
            }
            if report_has_field(report, "video_url") {
                message_html.push_str("<p><strong>Video Link:</strong> Please provide a valid URL for one of the supported video websites.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
