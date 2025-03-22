use std::error::Error;
use askama::Template;
use chrono::{ Datelike };
use garde::{ Report };

use crate::database::{ self, Band, AlbumType };
use crate::ui_modules::sidebar::{ SidebarTemplate, SidebarParams };
use crate::ui_primitives::alert::AlertTemplate;
use crate::router::routes::edit_album::{ EditAlbumPageContext };
use crate::router::validation::report_has_field;

struct SelectOption {
    value: String,
    text: String,
}

struct EditAlbumTemplateCommon<'a> {
    is_create: bool,
    has_access: bool,
    bands: Vec<Band>,
    album_types: Vec<AlbumType>,
    selected_band_slug: String,
    selected_album_slug: String,
    validation_alert: Option<AlertTemplate<'a>>,
    album_name: String,
    selected_album_type: String,
    publisher: String,
    years: Vec<SelectOption>,
    months: Vec<SelectOption>,
    days: Vec<SelectOption>,
    release_year: String,
    release_month: String,
    release_day_of_month: String,
    songs: Vec<String>,
    temporary_cover_picture_filename: String,
    cover_picture_file_path: String,
}

#[derive(Template)]
#[template(path = "ui_pages/edit_album.html")]
pub struct EditAlbumPageTemplate<'a> {
    active_page: &'a str,
    content: EditAlbumTemplateCommon<'a>,
    sidebar: SidebarTemplate<'a, EditAlbumPageContext>,
}
impl<'a> EditAlbumPageTemplate<'a> {
    pub async fn new(
        context: &'a EditAlbumPageContext
    ) -> Result<EditAlbumPageTemplate<'a>, Box<dyn Error>> {
        let active_page = "";
        let sidebar = SidebarTemplate::new(SidebarParams { context }).await?;

        let content = create_common_params(context).await?;

        Ok(EditAlbumPageTemplate {
            active_page,
            content,
            sidebar,
        })
    }
}

#[derive(Template)]
#[template(path = "ui_pages/edit_album.html", block = "page_content")]
pub struct EditAlbumPageContentTemplate<'a> {
    content: EditAlbumTemplateCommon<'a>,
}
impl<'a> EditAlbumPageContentTemplate<'a> {
    pub async fn new(
        context: &'a EditAlbumPageContext
    ) -> Result<EditAlbumPageContentTemplate<'a>, Box<dyn Error>> {

        let content = create_common_params(context).await?;

        Ok(EditAlbumPageContentTemplate {
            content,
        })
    }
}

fn get_page_title<'a>(content: &EditAlbumTemplateCommon<'a>) -> &'a str {
    match content.is_create {
        true => "Create New Album",
        _ => "Edit Album",
    }
}

fn get_submit_action<'a>(content: &EditAlbumTemplateCommon<'a>) -> String {
    if content.is_create {
        return String::from("/editor/create/song-album/");
    }
    return format!("/editor/update/song-album/{}/{}/",
        content.selected_band_slug, content.selected_album_slug,
    );
}

fn get_cancel_href<'a>(content: &EditAlbumTemplateCommon<'a>) -> String {
    if content.is_create {
        return format!("/lyrics/{}/", content.selected_band_slug);
    }
    return format!("/lyrics/{}/{}/",
        content.selected_band_slug, content.selected_album_slug,
    );
}

async fn create_common_params<'a>(context: &'a EditAlbumPageContext) -> Result<EditAlbumTemplateCommon<'a>, Box<dyn Error>> {
    let bands = database::get_all_bands().await?;

    let is_create: bool = context.route_original_uri.path().starts_with("/editor/create");
    let mut has_access: bool = true;

    let selected_band_slug = if context.params.band.is_empty() { "supercar" } else { &context.params.band };
    let selected_album_slug = &context.params.album;

    let validation_alert = get_validation_alert(&context.params.validation_report);
    if let Some(report) = &context.params.validation_report {
        if report_has_field(report, "forbidden") {
            has_access = false;
        }
    }

    let (
        album_name, selected_album_type, publisher, release_year, release_month, release_day_of_month,
        songs, temporary_cover_picture_filename, cover_picture_file_path,
    ) = if is_create || validation_alert.is_some() {
        let cover_picture_file_path = if !context.params.temporary_cover_picture_filename.is_empty() {
            format!("/assets/images/tmp/{}", &context.params.temporary_cover_picture_filename)
        } else {
            String::from("")
        };
        let mut songs: Vec<String> = context.params.songs
            .split(',')
            .map(|s| s.replace("%2C", ","))
            .map(|song| {
                song.to_string()
            })
            .collect::<Vec<_>>();
        songs.resize(40, String::from(""));
        (
            context.params.album_name.clone(),
            context.params.album_type.clone(),
            context.params.publisher.clone(),
            context.params.release_year.clone(),
            context.params.release_month.clone(),
            context.params.release_day_of_month.clone(),
            songs,
            context.params.temporary_cover_picture_filename.clone(),
            cover_picture_file_path,
        )
    } else {
        let selected_band = bands
            .iter()
            .find(|&band| &band.band_slug == selected_band_slug);
        let selected_band_id = if let Some(selected_band) = selected_band {
            selected_band.id
        } else {
            -1
        };
        let album = database::get_album_by_slug_and_band_id(&context.params.album, selected_band_id).await?;
        let mut songs = database::get_song_slugs_by_ids(&album.song_ids()).await?
            .into_iter()
            .map(|song| {
                song.song_name
            })
            .collect::<Vec<_>>();
        songs.resize(40, String::from(""));
        (
            album.album_name,
            album.album_type.as_key().to_string(),
            album.publisher,
            format!("{}", album.release_day.year()),
            format!("{}", album.release_day.month()),
            format!("{}", album.release_day.day()),
            songs,
            String::from(""),
            format!("/assets/images/album-covers/{}", album.cover_picture_filename),
        )
    };

    let mut years: Vec<SelectOption> = Vec::new();
    for year in (1900..chrono::Utc::now().year() + 1).rev() {
        years.push(SelectOption { value: year.to_string(), text: year.to_string() });
    }
    let months = vec!(
        SelectOption { value: "1".to_string(), text: "January".to_string() },
        SelectOption { value: "2".to_string(), text: "February".to_string() },
        SelectOption { value: "3".to_string(), text: "March".to_string() },
        SelectOption { value: "4".to_string(), text: "April".to_string() },
        SelectOption { value: "5".to_string(), text: "May".to_string() },
        SelectOption { value: "6".to_string(), text: "June".to_string() },
        SelectOption { value: "7".to_string(), text: "July".to_string() },
        SelectOption { value: "8".to_string(), text: "August".to_string() },
        SelectOption { value: "9".to_string(), text: "September".to_string() },
        SelectOption { value: "10".to_string(), text: "October".to_string() },
        SelectOption { value: "11".to_string(), text: "November".to_string() },
        SelectOption { value: "12".to_string(), text: "December".to_string() },
    );
    let mut days: Vec<SelectOption> = Vec::with_capacity(31);
    for day in 1..32 {
        days.push(SelectOption { value: day.to_string(), text: day.to_string() });
    }
    
    

    Ok(
        EditAlbumTemplateCommon {
            is_create,
            has_access,

            bands,
            album_types: AlbumType::to_values(),
            selected_band_slug: selected_band_slug.to_string(),
            selected_album_slug: selected_album_slug.to_string(),
            validation_alert,
            album_name,
            selected_album_type,
            publisher,
            years,
            months,
            days,
            release_year,
            release_month,
            release_day_of_month,
            songs,
            temporary_cover_picture_filename,
            cover_picture_file_path,
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
            if report_has_field(report, "album_exist") {
                message_html.push_str("<p>Another album with the same name already exists.</p>");
            }
            if report_has_field(report, "album_name") {
                message_html.push_str("<p><strong>Title:</strong> This field is required.</p>");
            }
            if report_has_field(report, "album_type") {
                message_html.push_str("<p><strong>Type:</strong> Invalid album type.</p>");
            }
            if report_has_field(report, "publisher") {
                message_html.push_str("<p><strong>Publisher:</strong> This field is required.</p>");
            }
            if report_has_field(report, "release_month") {
                message_html.push_str("<p><strong>Release Month:</strong> Invalid month specified.</p>");
            }
            if report_has_field(report, "release_day_of_month") {
                message_html.push_str("<p><strong>Release Day:</strong> Invalid day specified.</p>");
            }
            if report_has_field(report, "release_year") {
                message_html.push_str("<p><strong>Release Year:</strong> Invalid year specified.</p>");
            }
            if report_has_field(report, "temporary_cover_picture_filename") {
                message_html.push_str("<p><strong>Cover Image:</strong> Please upload a jpeg or png file that is less than 6 megabytes large.</p>");
            }
            if report_has_field(report, "image_transfer") {
                message_html.push_str("<p>Image upload failed. Please notify the site admins if this continues to happen.</p>");
            }

            Some(AlertTemplate {
                variant: "danger",
                message_html,
            })
        },
        _ => None,
    }
}
