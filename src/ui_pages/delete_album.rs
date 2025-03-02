use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_album::{ DeleteAlbumPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeleteAlbumTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    band: &'a str,
    album: &'a str,
    album_name: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_album.html")]
pub struct DeleteAlbumPageTemplate<'a> {
    active_page: &'a str,
    content: DeleteAlbumTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeleteAlbumPageContext>,
}
impl<'a> DeleteAlbumPageTemplate<'a> {
    pub async fn new(
        context: &'a DeleteAlbumPageContext
    ) -> Result<DeleteAlbumPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeleteAlbumPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_album.html", block = "page_content")]
pub struct DeleteAlbumPageContentTemplate<'a> {
    content: DeleteAlbumTemplateCommon<'a>,
}
impl<'a> DeleteAlbumPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeleteAlbumPageContext
    ) -> Result<DeleteAlbumPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeleteAlbumPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeleteAlbumTemplateCommon<'a>) -> String {
    return format!("/editor/delete/song-album/{}/{}/",
        content.band, content.album,
    );
}

async fn create_common_params<'a>(context: &'a DeleteAlbumPageContext) -> Result<DeleteAlbumTemplateCommon<'a>, Box<dyn Error>> {

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

    let mut album_name = String::from("");
    let album = if let Ok(album) = database::get_album_by_slug_and_band_id(&context.params.album, selected_band_id).await {
        album_name = album.album_name.clone();
        Some(album)
    } else {
        None
    };

    let album_missing_report = Some(
        create_simple_report(String::from("album_missing"), String::from("Album missing."))
    );
    let validation_alert = get_validation_alert(
        if album.is_some() {
            &context.params.validation_report
        } else {
            &album_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeleteAlbumTemplateCommon {
            has_access,
            validation_alert,
            band: &context.params.band,
            album: &context.params.album,
            album_name,
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
            if report_has_field(report, "album_missing") {
                message_html.push_str("<p>The specified album does not exist.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
