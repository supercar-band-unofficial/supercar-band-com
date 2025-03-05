use axum::{
    http::{ header, StatusCode },
    response::{ IntoResponse },
};

pub async fn get_robots() -> impl IntoResponse {
    let body = format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        "User-agent: *",
        "Disallow: /bio.php",
        "Disallow: /communityGuidelines.php",
        "Disallow: /lyrics.php",
        "Disallow: /members.php",
        "Disallow: /photos.php",
        "Disallow: /privacyPolicy.php",
        "Disallow: /signup.php",
        "Disallow: /tabs.php",
        "Disallow: /termsOfService.php",
        "Disallow: /videos.php",
        "Disallow: /404/",
        "Disallow: /assets/",
        "Disallow: /editor/",
    );

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/plain"),
        ],
        body,
    )
}
