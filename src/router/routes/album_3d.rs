use axum::{
    extract::Path,
    http::{ header, StatusCode },
    response::{ IntoResponse },
};

use crate::database;

pub async fn get_album_3d(
    Path((band, album)): Path<(String, String)>,
) -> impl IntoResponse {

    let assets = database::get_album_3d_assets(&band, &album).await;

    if assets.len() > 0 {
        let mut asset_json: Vec<String> = Vec::new();
        for (key, value) in assets.iter() {
            asset_json.push(
                format!(r#""{}":"{}""#, key, value)
            );
        }
        let body = format!(r#"{{"textures":{{{}}}}}"#, asset_json.join(","));
    
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/json"),
            ],
            body,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [
                (header::CONTENT_TYPE, "text/plain"),
            ],
            String::from(""),
        )
    }
}
