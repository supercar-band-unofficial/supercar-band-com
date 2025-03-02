use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self, SongTabType };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::util::format;
use crate::router::routes::delete_tabs::{ DeleteTabsPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeleteTabsTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    band: &'a str,
    song: &'a str,
    tab_type: String,
    contributor: String,
    song_name: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_tabs.html")]
pub struct DeleteTabsPageTemplate<'a> {
    active_page: &'a str,
    content: DeleteTabsTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeleteTabsPageContext>,
}
impl<'a> DeleteTabsPageTemplate<'a> {
    pub async fn new(
        context: &'a DeleteTabsPageContext
    ) -> Result<DeleteTabsPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeleteTabsPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_tabs.html", block = "page_content")]
pub struct DeleteTabsPageContentTemplate<'a> {
    content: DeleteTabsTemplateCommon<'a>,
}
impl<'a> DeleteTabsPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeleteTabsPageContext
    ) -> Result<DeleteTabsPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeleteTabsPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeleteTabsTemplateCommon<'a>) -> String {
    return format!("/editor/delete/tabs/{}/{}/{}/{}",
        content.band, content.song, content.tab_type, content.contributor
    );
}

async fn create_common_params<'a>(context: &'a DeleteTabsPageContext) -> Result<DeleteTabsTemplateCommon<'a>, Box<dyn Error>> {

    let mut has_access: bool = true;

    let bands = database::get_all_bands().await?;
    let selected_band_slug = &context.params.band;
    let selected_band = bands
        .iter()
        .find(|&band| &band.band_slug == selected_band_slug);
    let selected_band_id = if let Some(selected_band) = selected_band {
        selected_band.id
    } else {
        -1
    };
    
    let contributor = if context.params.contributor.is_empty() {
        if let Some(user) = &context.user {
            user.username.clone()
        } else {
            String::from("")    
        }
    } else {
        context.params.contributor.clone()
    };

    let song = database::get_song_by_slug_and_band_id(&context.params.song, selected_band_id).await?;
    let tabs = if let Ok(tabs) = database::get_song_tab_by_username_type_and_song_id(
        &contributor,
        &format::to_snake_case(&context.params.tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown),
        song.id
    ).await {
        Some(tabs)
    } else {
        None
    };

    let tabs_missing_report = Some(
        create_simple_report(String::from("tabs_missing"), String::from("Tabs missing."))
    );
    let validation_alert = get_validation_alert(
        if tabs.is_some() {
            &context.params.validation_report
        } else {
            &tabs_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeleteTabsTemplateCommon {
            has_access,
            validation_alert,
            band: &context.params.band,
            song: &context.params.song,
            tab_type: context.params.tab_type.clone(),
            contributor,
            song_name: song.song_name.clone(),
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
            if report_has_field(report, "tab_missing") {
                message_html.push_str("<p>The specified tab does not exist.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
