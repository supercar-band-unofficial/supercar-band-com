use std::error::Error;
use askama::Template;
use garde::{ Report };

use crate::database::{ self, AlbumSummary, Band, JoinedSongSlugs, Lyrics, Song };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::edit_lyrics::{ EditLyricsPageContext };
use crate::router::validation::report_has_field;

struct EditLyricsTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    bands: Vec<Band>,
    selected_band_slug: String,
    albums: Vec<AlbumSummary>,
    selected_album_slug: String,
    songs: Vec<JoinedSongSlugs>,
    selected_song_slug: String,
    no_songs_alert: Option<AlertTemplate<'a>>,
    validation_alert: Option<AlertTemplate<'a>>,
    kanji: String,
    romaji: String,
    english: String,
    comment: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_lyrics.html")]
pub struct EditLyricsPageTemplate<'a> {
    active_page: &'a str,
    content: EditLyricsTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditLyricsPageContext>,
}
impl<'a> EditLyricsPageTemplate<'a> {
    pub async fn new(
        context: &'a EditLyricsPageContext
    ) -> Result<EditLyricsPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditLyricsPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_lyrics.html", block = "page_content")]
pub struct EditLyricsPageContentTemplate<'a> {
    content: EditLyricsTemplateCommon<'a>,
}
impl<'a> EditLyricsPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditLyricsPageContext
    ) -> Result<EditLyricsPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditLyricsPageContentTemplate {
            content,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_lyrics.html", block = "select_band_album_song")]
pub struct EditLyricsSelectBandAlbumSongTemplate<'a> {
    content: EditLyricsTemplateCommon<'a>,
}
impl<'a> EditLyricsSelectBandAlbumSongTemplate<'a> {
    pub async fn new(
        context: &'a EditLyricsPageContext
    ) -> Result<EditLyricsSelectBandAlbumSongTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditLyricsSelectBandAlbumSongTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditLyricsTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Create Lyrics",
        _ => "Edit Lyrics",
    }
}

fn get_submit_action<'a>(content: &EditLyricsTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/lyrics/");
    }
    return format!("/editor/update/lyrics/{}/{}/{}/",
        content.selected_band_slug, content.selected_album_slug, content.selected_song_slug
    );
}

fn create_cancel_href<'a>(content: &EditLyricsTemplateCommon<'a>) -> String {
    if content.is_create {
        if !content.selected_song_slug.is_empty() && !content.selected_album_slug.is_empty() {
            return format!("/lyrics/{}/{}/{}/", content.selected_band_slug, content.selected_album_slug, content.selected_song_slug);
        } else if !content.selected_album_slug.is_empty() {
            return format!("/lyrics/{}/{}/", content.selected_band_slug, content.selected_album_slug);
        }
        return format!("/lyrics/{}/", content.selected_band_slug);
    }
    return format!("/lyrics/{}/{}/{}/", content.selected_band_slug, content.selected_album_slug, content.selected_song_slug);
}

async fn create_common_params<'a>(context: &'a EditLyricsPageContext) -> Result<EditLyricsTemplateCommon<'a>, Box<dyn Error>> {
    let bands = database::get_all_bands().await?;

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_band_slug = if context.params.band.is_empty() { "supercar" } else { &context.params.band };

    let selected_band_id = bands
        .iter()
        .find(|&band| band.band_slug == selected_band_slug)
        .unwrap()
        .id;

    let albums = database::get_album_summaries_by_band_id(selected_band_id).await?;
    let selected_album_slug: String = if context.params.album.is_empty() {
        albums.first().unwrap().album_slug.to_string()
    } else {
        context.params.album.to_string()
    };
    let album = database::get_album_by_slug_and_band_id(&selected_album_slug, selected_band_id).await?;

    let songs = database::get_song_slugs_by_ids(&album.song_ids()).await?;
    let selected_song_slug: String = if context.params.song.is_empty() {
        if let Some(first_song) = songs.first() {
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
                message_html: String::from("The selected album has no songs. Edit the album first to add songs."),
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

    let (kanji, romaji, english, comment) = if is_create || validation_alert.is_some() {
        (
            context.params.kanji.to_string(),
            context.params.romaji.to_string(),
            context.params.english.to_string(),
            context.params.comment.to_string(),
        )
    } else if context.user.is_some() {
        let Song { id: song_id, .. } = database::get_song_by_slug_and_band_id(&selected_song_slug, selected_band_id).await?;
        let lyrics: Lyrics = database::get_lyrics_by_username_and_song_id(&context.user.as_ref().unwrap().username, song_id).await.unwrap();
        (
            lyrics.kanji_content.to_string(),
            lyrics.romaji_content.to_string(),
            lyrics.english_content.to_string(),
            lyrics.comment.to_string(),
        )
    } else {
        ("".to_string(), "".to_string(), "".to_string(), "".to_string())
    };

    Ok(
        EditLyricsTemplateCommon {
            is_create,
            has_access,
            bands,
            selected_band_slug: selected_band_slug.to_string(),
            albums,
            selected_album_slug,
            songs,
            selected_song_slug: selected_song_slug.to_string(),
            validation_alert,
            no_songs_alert,
            kanji,
            romaji,
            english,
            comment,
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
            if report_has_field(report, "lyrics_exist") {
                message_html.push_str("<p>You have already created a translation for this song. Please edit the existing translation.</p>");
            }
            if report_has_field(report, "song_missing") {
                message_html.push_str("<p>The selected song doesn't exist. It may have been deleted after visiting this page.</p>");
            }
            if report_has_field(report, "kanji") {
                message_html.push_str("<p><strong>Kanji:</strong> This field is required.</p>");
            }
            if report_has_field(report, "romaji") {
                message_html.push_str("<p><strong>Rōmaji:</strong> This field is required.</p>");
            }
            if report_has_field(report, "english") {
                message_html.push_str("<p><strong>English:</strong> This field is required.</p>");
            }
            if report_has_field(report, "lines_mismatch") {
                message_html.push_str("<p>Kanji, Rōmaji, and English must have the same number of lines. Please double check that the line numbers match up.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
