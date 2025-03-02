use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self, Band, JoinedSongSlugs, Song, SongTab, SongTabType };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::util::format;
use crate::router::routes::edit_tabs::{ EditTabsPageContext };
use crate::router::validation::report_has_field;

struct EditTabsTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    bands: Vec<Band>,
    contributor: String,
    selected_band_slug: String,
    songs: Vec<JoinedSongSlugs>,
    is_song_slug_defaulted: bool,
    selected_song_slug: String,
    no_songs_alert: Option<AlertTemplate<'a>>,
    validation_alert: Option<AlertTemplate<'a>>,
    selected_tab_type: String,
    tab_types: Vec<SongTabType>,
    tab_content: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_tabs.html")]
pub struct EditTabsPageTemplate<'a> {
    active_page: &'a str,
    content: EditTabsTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditTabsPageContext>,
}
impl<'a> EditTabsPageTemplate<'a> {
    pub async fn new(
        context: &'a EditTabsPageContext
    ) -> Result<EditTabsPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditTabsPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_tabs.html", block = "page_content")]
pub struct EditTabsPageContentTemplate<'a> {
    content: EditTabsTemplateCommon<'a>,
}
impl<'a> EditTabsPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditTabsPageContext
    ) -> Result<EditTabsPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditTabsPageContentTemplate {
            content,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_tabs.html", block = "select_band_song")]
pub struct EditTabsSelectBandSongTemplate<'a> {
    content: EditTabsTemplateCommon<'a>,
}
impl<'a> EditTabsSelectBandSongTemplate<'a> {
    pub async fn new(
        context: &'a EditTabsPageContext
    ) -> Result<EditTabsSelectBandSongTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditTabsSelectBandSongTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditTabsTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Create Tabs",
        _ => "Edit Tabs",
    }
}

fn get_submit_action<'a>(content: &EditTabsTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/tabs/");
    }
    return format!("/editor/update/tabs/{}/{}/{}/{}/",
        content.selected_band_slug, content.selected_song_slug, format::to_kebab_case(&content.selected_tab_type), content.contributor,
    );
}

fn create_cancel_href<'a>(content: &EditTabsTemplateCommon<'a>) -> String {
    if content.is_create {
        if !content.is_song_slug_defaulted && !content.selected_song_slug.is_empty() {
            return format!("/tabs/{}/{}/", content.selected_band_slug, content.selected_song_slug);
        }
        return format!("/tabs/{}/", content.selected_band_slug);
    }
    return format!("/tabs/{}/{}/{}/{}/", content.selected_band_slug, content.selected_song_slug, format::to_kebab_case(&content.selected_tab_type), content.contributor);
}

async fn create_common_params<'a>(context: &'a EditTabsPageContext) -> Result<EditTabsTemplateCommon<'a>, Box<dyn Error>> {
    let bands = database::get_all_bands().await?;

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_band_slug = if context.params.band.is_empty() { "supercar" } else { &context.params.band };

    let selected_band_id = bands
        .iter()
        .find(|&band| band.band_slug == selected_band_slug)
        .unwrap()
        .id;

    let songs = database::get_song_slugs_by_band_id(selected_band_id).await?;
    let mut is_song_slug_defaulted = false;
    let selected_song_slug: String = if context.params.song.is_empty() {
        if let Some(first_song) = songs.first() {
            is_song_slug_defaulted = true;
            first_song.song_slug.to_string()
        } else {
            String::from("")
        }
    } else {
        context.params.song.to_string()
    };
    
    let no_songs_alert = if selected_song_slug.is_empty() {
        Some(
            AlertTemplate {
                variant: "danger",
                message_html: String::from("The selected band has no songs. Edit the band first to add songs."),
            }
        )
    } else {
        None
    };

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (tab_type, tab_content) = if is_create || validation_alert.is_some() {
        (
            context.params.tab_type.to_string(),
            context.params.tab_content.to_string(),
        )
    } else if context.user.is_some() {
        let Song { id: song_id, .. } = database::get_song_by_slug_and_band_id(&selected_song_slug, selected_band_id).await?;
        let tab: SongTab = database::get_song_tab_by_username_type_and_song_id(
            if context.params.contributor.is_empty() { &context.user.as_ref().unwrap().username } else { &context.params.contributor },
            &format::to_snake_case(&context.params.tab_type).parse::<SongTabType>().unwrap_or_else(|_| SongTabType::Unknown),
            song_id
        ).await?;
        (
            tab.tab_type.to_string(),
            tab.tab_content.to_string(),
        )
    } else {
        ("".to_string(), "".to_string())
    };

    Ok(
        EditTabsTemplateCommon {
            is_create,
            has_access,
            bands,
            contributor: context.params.contributor.clone(),
            selected_band_slug: selected_band_slug.to_string(),
            songs,
            is_song_slug_defaulted,
            selected_song_slug: selected_song_slug.to_string(),
            validation_alert,
            no_songs_alert,
            selected_tab_type: format::to_snake_case(&tab_type),
            tab_types: SongTabType::to_values(),
            tab_content,
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
            if report_has_field(report, "tab_exist") {
                message_html.push_str("<p>You have already created tabs for this song (and instrument). Please edit the existing tabs.</p>");
            }
            if report_has_field(report, "tab_missing") {
                message_html.push_str("<p>The selected song doesn't exist. It may have been deleted after visiting this page.</p>");
            }
            if report_has_field(report, "tab_content") {
                message_html.push_str("<p>Please enter the tabs.</p>");
            }
            if report_has_field(report, "song") {
                message_html.push_str("<p>No song is selected.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
