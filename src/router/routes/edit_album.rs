use std::error::Error;

use axum::{
    http::{ StatusCode },
    response::{ IntoResponse, Response, Redirect },
};
use askama::Template;
use chrono::NaiveDate;
use garde::{ Validate, Report };
use macros::{ RouteParamsContext, render_template };

use crate::database::{ self, Album, AlbumType, UserPermission, UserPreference };
use crate::router::context::{ BaseContext, Context, RouteParamContextGenerator };
use crate::ui_pages::edit_album::{ EditAlbumPageTemplate, EditAlbumPageContentTemplate };
use crate::util::format;
use crate::util::image_upload;
use crate::router::{ html_to_response };
use crate::router::validation::create_simple_report;

#[derive(Default, Debug, RouteParamsContext)]
pub struct EditAlbumPageParams {
    #[route_param_source(source = "none")]
    pub validation_report: Option<Report>,

    #[route_param_source(source = "path", name = "band", default = "")]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    pub album: String,

    #[route_param_source(default = "")]
    pub album_name: String,

    #[route_param_source(default = "")]
    pub album_type: String,

    #[route_param_source(default = "")]
    pub publisher: String,

    #[route_param_source(default = "")]
    pub release_year: String,
    
    #[route_param_source(default = "")]
    pub release_month: String,

    #[route_param_source(default = "")]
    pub release_day_of_month: String,

    #[route_param_source(default = "")]
    pub songs: String,

    #[route_param_source(default = "")]
    pub temporary_cover_picture_filename: String,
}
pub type EditAlbumPageContext = BaseContext<EditAlbumPageParams>;

pub async fn get_edit_album(
    Context { mut context }: Context<EditAlbumPageParams>,
) -> Response {

    let has_permissions = match &context.user {
        Some(user) => {
            user.permissions.contains(&UserPermission::CreateAlbum)
            || user.permissions.contains(&UserPermission::EditAlbum)
        },
        None => false,
    };
    if !has_permissions {
        context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
    }

    html_to_response(
        &context,
        |hx_target, context| async move {
            match hx_target.as_str() {
                "main-article" => render_template!(EditAlbumPageContentTemplate, &context),
                _ => render_template!(EditAlbumPageTemplate, &context),
            }
        }
    ).await
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct CreateAlbumPageParams {
    #[route_param_source(source = "form", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "form", name = "album-name", default = "")]
    #[garde(
        length(min = 1, max = 100),
    )]
    pub album_name: String,

    #[route_param_source(source = "form", name = "album-type", default = "")]
    #[garde(
        length(min = 1, max = 50),
        custom(is_valid_album_type(&self.album_type))
    )]
    pub album_type: String,

    #[route_param_source(source = "form", name = "publisher", default = "")]
    #[garde(
        length(min = 1, max = 100),
    )]
    pub publisher: String,

    #[route_param_source(source = "form", name = "release-year", default = "")]
    #[garde(
        length(max = 6),
    )]
    pub release_year: String,

    #[route_param_source(source = "form", name = "release-month", default = "")]
    #[garde(
        length(max = 2),
    )]
    pub release_month: String,

    #[route_param_source(source = "form", name = "release-day-of-month", default = "")]
    #[garde(
        length(max = 2),
        custom(is_valid_day_of_month(&self.release_year, &self.release_month, &self.release_day_of_month)),
    )]
    pub release_day_of_month: String,

    #[route_param_source(source = "form", name = "cover-image", default = "")]
    #[garde(skip)]
    pub cover_picture_upload: String,

    #[route_param_source(source = "form", name = "temporary-cover-image", default = "")]
    #[garde(
        custom(is_valid_image_upload(&self.temporary_cover_picture_filename, &self.cover_picture_upload)),
    )]
    pub temporary_cover_picture_filename: String,
}

#[axum::debug_handler]
pub async fn post_create_album(
    Context { context }: Context<CreateAlbumPageParams>,
) -> Response {

    let temporary_cover_picture_filename = if context.params.cover_picture_upload.is_empty() {
        context.params.temporary_cover_picture_filename.clone()
    } else {
        context.params.cover_picture_upload.clone()
    };

    let mut page_context = context.clone_with_params(EditAlbumPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        album: String::from(""),
        album_name: context.params.album_name.clone(),
        album_type: context.params.album_type.clone(),
        publisher: context.params.publisher.clone(),
        release_year: context.params.release_year.clone(),
        release_month: context.params.release_month.clone(),
        release_day_of_month: context.params.release_day_of_month.clone(),
        songs: String::from(""),
        temporary_cover_picture_filename: temporary_cover_picture_filename.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::CreateAlbum),
        None => false,
    };
    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_album_create_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_album_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }

    let album_slug = format::to_kebab_case(&context.params.album_name);
    let band_id: i32;
    match validate_album_dont_exist(&album_slug, &context.params.band).await {
        Ok(band_id_attr) => {
            band_id = band_id_attr;
        },
        Err(_) => {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("album_exist"), String::from("User already created album."))
            );
            return send_edit_album_page_response(StatusCode::CONFLICT, page_context).await;
        },
    }

    let mut permanent_filename = format!("{}-{}", context.params.band, album_slug);
    match image_upload::transfer_temporary_image_upload(
        &temporary_cover_picture_filename,
        "album-covers",
        &permanent_filename
    ).await {
        Ok(permanent_filename_with_extension) => {
            permanent_filename = permanent_filename_with_extension;
        },
        Err(_) => {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("image_transfer"), String::from("Image upload failed."))
            );
            return send_edit_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
        }
    }

    let user = context.user.unwrap();
    let username = user.username;

    let album = Album {
        username: username.clone(),
        band: band_id,
        album_slug: album_slug.clone(),
        album_name: context.params.album_name.clone(),
        album_type: AlbumType::from(context.params.album_type.as_str()),
        publisher: context.params.publisher,
        cover_picture_filename: permanent_filename,
        release_day: NaiveDate::from_ymd_opt(
            context.params.release_year.parse::<i32>().unwrap_or_else(|_| 2000),
            context.params.release_month.parse::<u32>().unwrap_or_else(|_| 1),
            context.params.release_day_of_month.parse::<u32>().unwrap_or_else(|_| 1)
        ).unwrap_or_else(|| NaiveDate::default()),
        ..Album::default()
    };

    if let Err(error) = database::create_album(album).await {
        tracing::warn!("Database call failed when user {} tried to create album. {:?}", &username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    }

    if user.preferences.contains(&UserPreference::NotifyGlobalFeed) {
        let _ = database::notify_album_created(
            &username,
            &context.params.band,
            &album_slug,
            &context.params.album_name,
        ).await;
    }

    Redirect::to(
        format!("/lyrics/{}/{}/", context.params.band, &album_slug).as_str()
    ).into_response()
}

async fn validate_album_dont_exist(album_slug: &str, band_slug: &str) -> Result<i32, bool> {
    let band_id = if let Ok(band) = database::get_band_by_slug(band_slug).await {
        band.id
    } else {
        -1
    };
    if let Ok(_) = database::get_album_by_slug_and_band_id(album_slug, band_id).await {
        return Err(false);
    }
    Ok(band_id)
}

async fn validate_album_create_form(form: &CreateAlbumPageParams) -> Result<(), Report> {
    if let Err(report) = form.validate() {
        return Err(report);
    }
    Ok(())
}

#[derive(Default, Debug, RouteParamsContext, Validate)]
pub struct UpdateAlbumPageParams {
    #[route_param_source(source = "path", name = "band", default = "")]
    #[garde(skip)]
    pub band: String,

    #[route_param_source(source = "path", name = "album", default = "")]
    #[garde(skip)]
    pub album: String,

    #[route_param_source(source = "form", name = "album-name", default = "")]
    #[garde(
        length(min = 1, max = 100),
    )]
    pub album_name: String,

    #[route_param_source(source = "form", name = "album-type", default = "")]
    #[garde(
        length(min = 1, max = 50),
        custom(is_valid_album_type(&self.album_type))
    )]
    pub album_type: String,

    #[route_param_source(source = "form", name = "publisher", default = "")]
    #[garde(
        length(min = 1, max = 100),
    )]
    pub publisher: String,

    #[route_param_source(source = "form", name = "release-year", default = "")]
    #[garde(
        length(max = 6),
    )]
    pub release_year: String,

    #[route_param_source(source = "form", name = "release-month", default = "")]
    #[garde(
        length(max = 2),
    )]
    pub release_month: String,

    #[route_param_source(source = "form", name = "release-day-of-month", default = "")]
    #[garde(
        length(max = 2),
        custom(is_valid_day_of_month(&self.release_year, &self.release_month, &self.release_day_of_month)),
    )]
    pub release_day_of_month: String,

    #[route_param_source(source = "form", name = "songs", default = "")]
    #[garde(skip)]
    pub songs: String,

    #[route_param_source(source = "form", name = "cover-image", default = "")]
    #[garde(skip)]
    pub cover_picture_upload: String,

    #[route_param_source(source = "form", name = "temporary-cover-image", default = "")]
    #[garde(skip)]
    pub temporary_cover_picture_filename: String,
}

pub async fn put_update_album(
    Context { context }: Context<UpdateAlbumPageParams>,
) -> Response {

    let temporary_cover_picture_filename = if !context.params.temporary_cover_picture_filename.is_empty() {
        context.params.temporary_cover_picture_filename.clone()
    } else {
        context.params.cover_picture_upload.clone()
    };

    let mut page_context = context.clone_with_params(EditAlbumPageParams {
        validation_report: None,
        band: context.params.band.clone(),
        album: context.params.album.clone(),
        album_name: context.params.album_name.clone(),
        album_type: context.params.album_type.clone(),
        publisher: context.params.publisher.clone(),
        release_year: context.params.release_year.clone(),
        release_month: context.params.release_month.clone(),
        release_day_of_month: context.params.release_day_of_month.clone(),
        songs: context.params.songs.clone(),
        temporary_cover_picture_filename: temporary_cover_picture_filename.clone(),
    });

    let has_permissions = match &context.user {
        Some(user) => user.permissions.contains(&UserPermission::EditAlbum),
        None => false,
    };

    if !has_permissions {
        page_context.params.validation_report = Some(
            create_simple_report(String::from("forbidden"), String::from("Not Allowed."))
        );
        return send_edit_album_page_response(StatusCode::FORBIDDEN, page_context).await;
    }

    let validation_result = validate_album_update_form(&context.params).await;
    if let Err(report) = validation_result {
        page_context.params.validation_report = Some(report);
        return send_edit_album_page_response(StatusCode::BAD_REQUEST, page_context).await;
    }
    let mut existing_album = validation_result.unwrap();

    let album_slug = format::to_kebab_case(&context.params.album_name);
    if album_slug != existing_album.album_slug {
        if let Err(_) = validate_album_dont_exist(&album_slug, &context.params.band).await {
            page_context.params.validation_report = Some(
                create_simple_report(String::from("album_exist"), String::from("User already created album."))
            );
            return send_edit_album_page_response(StatusCode::CONFLICT, page_context).await;
        }
    }

    let user = &context.user.as_ref().unwrap();
    let username = &user.username;

    let mut songs: Vec<&str> = context.params.songs
        .split(',')
        .collect::<Vec<_>>();
    songs.resize(40, "");
    let song_ids: Vec<Option<i32>>;
    match database::create_songs_by_names(&songs, existing_album.id, existing_album.band).await {
        Ok(ids) => {
            song_ids = ids;
        },
        Err(error) => {
            tracing::warn!("Database call failed when user {} tried to create/match songs in an album. {:?}", username, error);
            page_context.params.validation_report = Some(
                create_simple_report(String::from("server_error"), String::from("Error while creating songs."))
            );
            return send_edit_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
        }
    }
    existing_album = existing_album.populate_song_ids(song_ids);

    existing_album.album_slug = album_slug.clone();
    existing_album.album_name = context.params.album_name.clone();
    existing_album.album_type = AlbumType::from(context.params.album_type.as_str());
    existing_album.publisher = context.params.publisher;
    existing_album.release_day = NaiveDate::from_ymd_opt(
        context.params.release_year.parse::<i32>().unwrap_or_else(|_| 2000),
        context.params.release_month.parse::<u32>().unwrap_or_else(|_| 1),
        context.params.release_day_of_month.parse::<u32>().unwrap_or_else(|_| 1)
    ).unwrap_or_else(|| NaiveDate::default());

    if !temporary_cover_picture_filename.is_empty() {
        let mut permanent_filename = format!("{}-{}", context.params.band, album_slug);
        match image_upload::transfer_temporary_image_upload(
            &temporary_cover_picture_filename,
            "album-covers",
            &permanent_filename
        ).await {
            Ok(permanent_filename_with_extension) => {
                permanent_filename = permanent_filename_with_extension;
            },
            Err(_) => {
                page_context.params.validation_report = Some(
                    create_simple_report(String::from("image_transfer"), String::from("Image upload failed."))
                );
                return send_edit_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
            }
        }
        existing_album.cover_picture_filename = permanent_filename;
    }

    if let Err(error) = database::update_album(existing_album).await {
        tracing::warn!("Database call failed when user {} tried to update album. {:?}", username, error);
        page_context.params.validation_report = Some(
            create_simple_report(String::from("server_error"), String::from("An error occurred with the request."))
        );
        return send_edit_album_page_response(StatusCode::INTERNAL_SERVER_ERROR, page_context).await;
    };

    Redirect::to(
        format!("/lyrics/{}/{}/", context.params.band, album_slug).as_str()
    ).into_response()
}

async fn validate_album_update_form(form: &UpdateAlbumPageParams) -> Result<Album, Report> {
    let validation_result = validate_album_exists(&form.band, &form.album).await;
    if let Err(_) = validation_result {
        return Err(
            create_simple_report(String::from("album_missing"), String::from("The specified album does not exist."))
        );
    }
    if let Err(report) = form.validate() {
        return Err(report);
    }
    let album = validation_result.unwrap();
    Ok(album)
}

async fn validate_album_exists(band_slug: &str, album_slug: &str) -> Result<Album, Box<dyn Error>> {
    let band = database::get_band_by_slug(band_slug).await?;
    let album = database::get_album_by_slug_and_band_id(album_slug, band.id).await?;
    Ok(album)
}

pub async fn send_edit_album_page_response(status: StatusCode, context: EditAlbumPageContext) -> Response {
    (
        status,
        html_to_response(
            &context,
            |hx_target, context| async move {
                match hx_target.as_str() {
                    "main-article" => render_template!(EditAlbumPageContentTemplate, &context),
                    _ => render_template!(EditAlbumPageTemplate, &context),
                }
            }
        ).await
    ).into_response()
}

fn is_valid_album_type(album_type: &str) -> impl FnOnce(&str, &()) -> garde::Result + '_ {
    move |_, _| {
        if AlbumType::from(album_type) != AlbumType::Unknown {
            Ok(())
        } else {
            Err(garde::Error::new("Invalid album type."))
        }
    }
}

fn is_valid_image_upload<'a>(new_filename: &'a str, existing_filename: &'a str) -> impl FnOnce(&str, &()) -> garde::Result + 'a {
    move |_, _| {
        if new_filename.is_empty() && existing_filename.is_empty() {
            Err(garde::Error::new("Missing file."))
        } else {
            Ok(())
        }
    }
}

fn is_valid_day_of_month<'a>(year: &'a str, month: &'a str, day: &'a str) -> impl FnOnce(&str, &()) -> garde::Result + 'a {
    move |_, _| {
        if let (Ok(y), Ok(m), Ok(d)) = (year.parse::<i32>(), month.parse::<u32>(), day.parse::<u32>()) {
            if NaiveDate::from_ymd_opt(y, m, d).is_some() {
                Ok(())
            } else {
                Err(garde::Error::new("Invalid date."))
            }
        } else {
            Err(garde::Error::new("Invalid date entry."))
        }
    }
}
