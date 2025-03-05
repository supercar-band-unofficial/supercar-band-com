use axum::{
    extract::Path,
    http::{ header, StatusCode },
    response::{ IntoResponse },
};
use serde_json;

use crate::database;

pub async fn get_album_3d(
    Path((band, album)): Path<(String, String)>,
) -> impl IntoResponse {

    let assets_and_config = database::get_album_3d_assets(&band, &album).await;

    if assets_and_config.assets.len() > 0 {
        let mut asset_json: Vec<String> = Vec::new();
        for (key, value) in assets_and_config.assets.iter() {
            asset_json.push(
                format!(r#""{}":"{}""#, key, value)
            );
        }
        let body = format!(
            r#"{{"config":{},"textures":{{{}}}}}"#,
            serde_json::to_string(&assets_and_config.config).unwrap(),
            asset_json.join(","),
        );
    
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
