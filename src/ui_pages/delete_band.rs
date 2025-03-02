use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_band::{ DeleteBandPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeleteBandTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    band: &'a str,
    band_name: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_band.html")]
pub struct DeleteBandPageTemplate<'a> {
    active_page: &'a str,
    content: DeleteBandTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeleteBandPageContext>,
}
impl<'a> DeleteBandPageTemplate<'a> {
    pub async fn new(
        context: &'a DeleteBandPageContext
    ) -> Result<DeleteBandPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeleteBandPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_band.html", block = "page_content")]
pub struct DeleteBandPageContentTemplate<'a> {
    content: DeleteBandTemplateCommon<'a>,
}
impl<'a> DeleteBandPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeleteBandPageContext
    ) -> Result<DeleteBandPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeleteBandPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeleteBandTemplateCommon<'a>) -> String {
    return format!("/editor/delete/band/{}/",
        content.band,
    );
}

async fn create_common_params<'a>(context: &'a DeleteBandPageContext) -> Result<DeleteBandTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = true;

    let selected_band_slug = &context.params.band;
    let band = database::get_band_by_slug(selected_band_slug).await;

    let band_missing_report = Some(
        create_simple_report(String::from("band_missing"), String::from("Album missing."))
    );
    let mut band_name: String = String::from("");
    let validation_alert = get_validation_alert(
        if let Ok(band) = band {
            band_name = band.band_name;
            &context.params.validation_report
        } else {
            &band_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeleteBandTemplateCommon {
            has_access,
            validation_alert,
            band: &context.params.band,
            band_name,
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
            if report_has_field(report, "band_missing") {
                message_html.push_str("<p>The specified band does not exist.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
