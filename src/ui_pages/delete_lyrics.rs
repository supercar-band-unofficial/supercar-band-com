use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::delete_lyrics::{ DeleteLyricsPageContext };
use crate::router::validation::{ report_has_field, create_simple_report };

struct DeleteLyricsTemplateCommon<'a> {
    has_access: bool,
    validation_alert: Option<AlertTemplate<'a>>,
    band: &'a str,
    album: &'a str,
    song: &'a str,
    contributor: String,
    song_name: String,
}

#[derive(Template)]
#[template(path = "ui_pages/delete_lyrics.html")]
pub struct DeleteLyricsPageTemplate<'a> {
    active_page: &'a str,
    content: DeleteLyricsTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, DeleteLyricsPageContext>,
}
impl<'a> DeleteLyricsPageTemplate<'a> {
    pub async fn new(
        context: &'a DeleteLyricsPageContext
    ) -> Result<DeleteLyricsPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(DeleteLyricsPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/delete_lyrics.html", block = "page_content")]
pub struct DeleteLyricsPageContentTemplate<'a> {
    content: DeleteLyricsTemplateCommon<'a>,
}
impl<'a> DeleteLyricsPageContentTemplate<'a> {
    pub async fn new(
        context: &'a DeleteLyricsPageContext
    ) -> Result<DeleteLyricsPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(DeleteLyricsPageContentTemplate {
            content,
        })
    }
}

fn get_submit_action<'a>(content: &DeleteLyricsTemplateCommon<'a>) -> String {
    return format!("/editor/delete/lyrics/{}/{}/{}/{}",
        content.band, content.album, content.song, content.contributor
    );
}

async fn create_common_params<'a>(context: &'a DeleteLyricsPageContext) -> Result<DeleteLyricsTemplateCommon<'a>, Box<dyn Error>> {

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
    let lyrics = if let Ok(lyrics) = database::get_lyrics_by_username_and_song_id(&contributor, song.id).await {
        Some(lyrics)
    } else {
        None
    };

    let lyrics_missing_report = Some(
        create_simple_report(String::from("lyrics_missing"), String::from("Lyrics missing."))
    );
    let validation_alert = get_validation_alert(
        if lyrics.is_some() {
            &context.params.validation_report
        } else {
            &lyrics_missing_report
        }
    );
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    Ok(
        DeleteLyricsTemplateCommon {
            has_access,
            validation_alert,
            band: &context.params.band,
            album: &context.params.album,
            song: &context.params.song,
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
            if report_has_field(report, "lyrics_missing") {
                message_html.push_str("<p>The specified translation does not exist.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
