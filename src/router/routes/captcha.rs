use axum::{
    body::Bytes,
    extract::Path,
    http::{ header, HeaderMap, StatusCode },
    response::IntoResponse,
};
use crate::util::captcha::{ get_captcha_image_by_id as get_captcha_image };

pub async fn get_captcha_image_by_id(
    Path(captcha_id): Path<String>,
) -> impl IntoResponse {

    let mut headers = HeaderMap::new();

    let response = match get_captcha_image(&captcha_id) {
        Some(png) => {
            headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("image/png"));
            headers.insert(header::CACHE_CONTROL, header::HeaderValue::from_static("max-age=180, public"));
            (StatusCode::OK, headers, Bytes::from(png))
        },
        _ => (StatusCode::NOT_FOUND, headers, Bytes::from("")),
    };

    response
}
