use axum::{
    http::{ header, StatusCode },
    response::{ IntoResponse },
};

use crate::database;
use crate::util::format::{ escape_xml_special_characters };

pub async fn get_sitemap() -> impl IntoResponse {
    let mut urls: Vec<String> = Vec::new();

    urls.push("https://supercarband.com/".to_string());
    urls.push("https://supercarband.com/bio/".to_string());
    urls.push("https://supercarband.com/community-guidelines/".to_string());
    urls.push("https://supercarband.com/lyrics/".to_string());
    let mut previous_band: String = String::from("");
    let mut previous_album: String = String::from("");
    for song in database::get_all_song_slugs().await.unwrap() {
        let song_name = escape_xml_special_characters(&song.song_slug);
        let album_name = escape_xml_special_characters(&song.album_slug);
        let band_name = escape_xml_special_characters(&song.band_slug);
        if band_name != previous_band {
            urls.push(format!("https://supercarband.com/lyrics/{}/", band_name));
            previous_band = band_name.clone();
        }
        if album_name != previous_album {
            urls.push(format!("https://supercarband.com/lyrics/{}/{}/", band_name, album_name));
            previous_album = album_name.clone();
        }
        urls.push(format!("https://supercarband.com/lyrics/{}/{}/{}/", band_name, album_name, song_name));
    }
    urls.push("https://supercarband.com/members/".to_string());
    urls.push("https://supercarband.com/photos/".to_string());
    for album in database::get_all_photo_albums().await.unwrap() {
        urls.push(format!("https://supercarband.com/photos/{}/", album.slug));
    }
    urls.push("https://supercarband.com/privacy-policy/".to_string());
    urls.push("https://supercarband.com/tabs/".to_string());
    for band in database::get_all_bands().await.unwrap() {
        urls.push(format!("https://supercarband.com/tabs/{}/", band.band_slug));
        for album in database::get_albums_by_band_id(band.id).await.unwrap() {
            if &album.album_type == &database::AlbumType::Full || &album.album_type == &database::AlbumType::Single {
                for song in database::get_song_slugs_by_ids(&album.song_ids()).await.unwrap() {
                    urls.push(format!("https://supercarband.com/tabs/{}/{}/", band.band_slug, song.song_slug));
                }
            }
        }
    }
    urls.push("https://supercarband.com/terms-of-service/".to_string());
    urls.push("https://supercarband.com/videos/".to_string());
    for category in database::get_all_video_categories().await.unwrap() {
        urls.push(format!("https://supercarband.com/videos/{}/", category.slug));
    }

    let urls_xml = urls
        .into_iter()
        .map(|url| format!("\n<url><loc>{}</loc></url>", url))
        .collect::<Vec<_>>()
        .join("");
    let body = format!(r#"<?xml version="1.0" encoding="UTF-8" ?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{}</urlset>"#, urls_xml);
    
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/xml"),
        ],
        body,
    )
}
