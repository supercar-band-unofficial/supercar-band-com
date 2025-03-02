use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::edit_band::{ EditBandPageContext };
use crate::router::validation::report_has_field;

struct EditBandTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    selected_band_slug: String,
    band_name: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_band.html")]
pub struct EditBandPageTemplate<'a> {
    active_page: &'a str,
    content: EditBandTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditBandPageContext>,
}
impl<'a> EditBandPageTemplate<'a> {
    pub async fn new(
        context: &'a EditBandPageContext
    ) -> Result<EditBandPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditBandPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_band.html", block = "page_content")]
pub struct EditBandPageContentTemplate<'a> {
    content: EditBandTemplateCommon<'a>,
}
impl<'a> EditBandPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditBandPageContext
    ) -> Result<EditBandPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditBandPageContentTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditBandTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Create Band",
        _ => "Edit Band",
    }
}

fn get_cancel_href<'a>(content: &EditBandTemplateCommon<'a>) -> String {
    if content.is_create {
        return format!("/lyrics/");
    }
    return format!("/lyrics/{}/",
        content.selected_band_slug,
    );
}

fn get_submit_action<'a>(content: &EditBandTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/band/");
    }
    return format!("/editor/update/band/{}/",
        content.selected_band_slug
    );
}

async fn create_common_params<'a>(context: &'a EditBandPageContext) -> Result<EditBandTemplateCommon<'a>, Box<dyn Error>> {

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_band_slug = &context.params.band;

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let band_name = if is_create || validation_alert.is_some() {
        context.params.band_name.clone()        
    } else {
        let band = database::get_band_by_slug(selected_band_slug).await?;
        band.band_name
    };

    Ok(
        EditBandTemplateCommon {
            is_create,
            has_access,
            validation_alert,
            band_name,
            selected_band_slug: selected_band_slug.to_string(),
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
            if report_has_field(report, "band_exist") {
                message_html.push_str("<p>A band with this name already exists. Please edit the existing band.</p>");
            }
            if report_has_field(report, "band_missing") {
                message_html.push_str("<p>The band doesn't exist. It may have been deleted after visiting this page.</p>");
            }
            if report_has_field(report, "band_name") {
                message_html.push_str("<p><strong>Band/Artist Name:</strong> This field is required.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
