use std::path::PathBuf;

use axum::{
    extract::{ DefaultBodyLimit },
    routing::{ delete, get, get_service, post, put },
    Router,
};
use axum_login::{
    tower_sessions::{ MemoryStore, SessionManagerLayer },
    AuthManagerLayerBuilder,
};
use memory_serve::{ load_assets, MemoryServe };
use tower_http::{
    services::{ ServeDir },
};
use super::authn::Backend;

pub mod bio;
pub mod captcha;
pub mod chat_box;
pub mod comment_section;
pub mod community_guidelines;
pub mod delete_album;
pub mod delete_band;
pub mod delete_lyrics;
pub mod delete_photo;
pub mod delete_photo_album;
pub mod delete_tabs;
pub mod delete_video;
pub mod delete_video_category;
pub mod edit_album;
pub mod edit_band;
pub mod edit_lyrics;
pub mod edit_photo;
pub mod edit_photo_album;
pub mod edit_profile_info;
pub mod edit_profile_password;
pub mod edit_profile_picture;
pub mod edit_tabs;
pub mod edit_video;
pub mod edit_video_category;
pub mod forgot_password;
pub mod home;
pub mod lyrics;
pub mod lyrics_booklet;
pub mod members;
pub mod page_not_found;
pub mod photos;
pub mod privacy_policy;
pub mod sign_in;
pub mod sign_out;
pub mod sign_up;
pub mod sitemap;
pub mod tabs;
pub mod terms_of_service;
pub mod videos;

pub fn initialize() -> Router {
    // Static assets
    let memory_router = MemoryServe::new(load_assets!("static"))
        .into_router();

    // Uploaded assets
    let uploaded_files_path = PathBuf::from("uploads");

    // Session layer.
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    // Auth service.
    let backend = Backend::default();
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        .merge(memory_router)
        .route("/", get(home::get_home))

        .route("/404", get(page_not_found::get_page_not_found))
        .route("/404/", get(page_not_found::get_page_not_found))

        .route("/bio", get(bio::get_bio))
        .route("/bio/", get(bio::get_bio))
        .route("/bio.php", get(bio::get_bio_redirect))

        .route("/captchas/{captcha_id}/captcha.png", get(captcha::get_captcha_image_by_id))

        .route("/chat-box", get(chat_box::get_chat_box))
        .route("/chat-box/", get(chat_box::get_chat_box))
        .route("/chat-box", post(chat_box::post_chat_box))
        .route("/chat-box/", post(chat_box::post_chat_box))

        .route("/comment-section/{section}/{section_tag_id}", get(comment_section::get_comment_section))
        .route("/comment-section/{section}/{section_tag_id}/", get(comment_section::get_comment_section))
        .route("/comment-section/{section}/{section_tag_id}/{reply_id}", get(comment_section::get_comment_section))
        .route("/comment-section/{section}/{section_tag_id}/{reply_id}/", get(comment_section::get_comment_section))
        .route("/comment-section/{section}/{section_tag_id}", post(comment_section::post_comment_section))
        .route("/comment-section/{section}/{section_tag_id}/", post(comment_section::post_comment_section))
        .route("/comment-section/{section}/{section_tag_id}/{reply_id}", post(comment_section::post_comment_section))
        .route("/comment-section/{section}/{section_tag_id}/{reply_id}/", post(comment_section::post_comment_section))

        .route("/community-guidelines", get(community_guidelines::get_community_guidelines))
        .route("/community-guidelines/", get(community_guidelines::get_community_guidelines))
        .route("/communityGuidelines.php", get(community_guidelines::get_community_guidelines_redirect))

        .route("/editor/create/band", get(edit_band::get_edit_band))
        .route("/editor/create/band/", get(edit_band::get_edit_band))
        .route("/editor/create/band", post(edit_band::post_create_band))
        .route("/editor/create/band/", post(edit_band::post_create_band))
        .route("/editor/delete/band/{band}", get(delete_band::get_delete_band))
        .route("/editor/delete/band/{band}/", get(delete_band::get_delete_band))
        .route("/editor/delete/band/{band}", delete(delete_band::delete_band))
        .route("/editor/delete/band/{band}/", delete(delete_band::delete_band))
        .route("/editor/delete/band/{band}", post(delete_band::delete_band))
        .route("/editor/delete/band/{band}/", post(delete_band::delete_band))
        .route("/editor/update/band/{band}", get(edit_band::get_edit_band))
        .route("/editor/update/band/{band}/", get(edit_band::get_edit_band))
        .route("/editor/update/band/{band}", put(edit_band::put_update_band))
        .route("/editor/update/band/{band}/", put(edit_band::put_update_band))
        .route("/editor/update/band/{band}", post(edit_band::put_update_band))
        .route("/editor/update/band/{band}/", post(edit_band::put_update_band))

        .route("/editor/create/lyrics", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/create/lyrics/", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/create/lyrics", post(edit_lyrics::post_create_lyrics))
        .route("/editor/create/lyrics/", post(edit_lyrics::post_create_lyrics))
        .route("/editor/create/lyrics/{band}", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/create/lyrics/{band}/", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/create/lyrics/{band}/{album}", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/create/lyrics/{band}/{album}/", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}", get(delete_lyrics::get_delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/", get(delete_lyrics::get_delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/{contributor}", get(delete_lyrics::get_delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/{contributor}/", get(delete_lyrics::get_delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/{contributor}", delete(delete_lyrics::delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/{contributor}/", delete(delete_lyrics::delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/{contributor}", post(delete_lyrics::delete_lyrics))
        .route("/editor/delete/lyrics/{band}/{album}/{song}/{contributor}/", post(delete_lyrics::delete_lyrics))
        .route("/editor/update/lyrics/{band}/{album}/{song}", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/update/lyrics/{band}/{album}/{song}/", get(edit_lyrics::get_edit_lyrics))
        .route("/editor/update/lyrics/{band}/{album}/{song}", post(edit_lyrics::put_update_lyrics))
        .route("/editor/update/lyrics/{band}/{album}/{song}/", post(edit_lyrics::put_update_lyrics))
        .route("/editor/update/lyrics/{band}/{album}/{song}", put(edit_lyrics::put_update_lyrics))
        .route("/editor/update/lyrics/{band}/{album}/{song}/", put(edit_lyrics::put_update_lyrics))

        .route("/editor/create/photo", get(edit_photo::get_edit_photo))
        .route("/editor/create/photo/", get(edit_photo::get_edit_photo))
        .route("/editor/create/photo", post(edit_photo::post_create_photo).layer(DefaultBodyLimit::max(1024 * 1024 * 12)))
        .route("/editor/create/photo/", post(edit_photo::post_create_photo).layer(DefaultBodyLimit::max(1024 * 1024 * 12)))
        .route("/editor/create/photo/{album}", get(edit_photo::get_edit_photo))
        .route("/editor/create/photo/{album}/", get(edit_photo::get_edit_photo))
        .route("/editor/delete/photo/{album}/{photo}", get(delete_photo::get_delete_photo))
        .route("/editor/delete/photo/{album}/{photo}/", get(delete_photo::get_delete_photo))
        .route("/editor/delete/photo/{album}/{photo}", delete(delete_photo::delete_photo))
        .route("/editor/delete/photo/{album}/{photo}/", delete(delete_photo::delete_photo))
        .route("/editor/delete/photo/{album}/{photo}", post(delete_photo::delete_photo))
        .route("/editor/delete/photo/{album}/{photo}/", post(delete_photo::delete_photo))
        .route("/editor/update/photo/{album}/{photo}", get(edit_photo::get_edit_photo))
        .route("/editor/update/photo/{album}/{photo}/", get(edit_photo::get_edit_photo))
        .route("/editor/update/photo/{album}/{photo}", put(edit_photo::put_update_photo))
        .route("/editor/update/photo/{album}/{photo}/", put(edit_photo::put_update_photo))
        .route("/editor/update/photo/{album}/{photo}", post(edit_photo::put_update_photo))
        .route("/editor/update/photo/{album}/{photo}/", post(edit_photo::put_update_photo))

        .route("/editor/create/photo-album", get(edit_photo_album::get_edit_photo_album))
        .route("/editor/create/photo-album/", get(edit_photo_album::get_edit_photo_album))
        .route("/editor/create/photo-album", post(edit_photo_album::post_create_photo_album))
        .route("/editor/create/photo-album/", post(edit_photo_album::post_create_photo_album))
        .route("/editor/delete/photo-album/{album}", get(delete_photo_album::get_delete_photo_album))
        .route("/editor/delete/photo-album/{album}/", get(delete_photo_album::get_delete_photo_album))
        .route("/editor/delete/photo-album/{album}", delete(delete_photo_album::delete_photo_album))
        .route("/editor/delete/photo-album/{album}/", delete(delete_photo_album::delete_photo_album))
        .route("/editor/delete/photo-album/{album}", post(delete_photo_album::delete_photo_album))
        .route("/editor/delete/photo-album/{album}/", post(delete_photo_album::delete_photo_album))
        .route("/editor/update/photo-album/{album}", get(edit_photo_album::get_edit_photo_album))
        .route("/editor/update/photo-album/{album}/", get(edit_photo_album::get_edit_photo_album))
        .route("/editor/update/photo-album/{album}", put(edit_photo_album::put_update_photo_album))
        .route("/editor/update/photo-album/{album}/", put(edit_photo_album::put_update_photo_album))
        .route("/editor/update/photo-album/{album}", post(edit_photo_album::put_update_photo_album))
        .route("/editor/update/photo-album/{album}/", post(edit_photo_album::put_update_photo_album))

        .route("/editor/update/profile-info", get(edit_profile_info::get_edit_profile_info))
        .route("/editor/update/profile-info/", get(edit_profile_info::get_edit_profile_info))
        .route("/editor/update/profile-info", put(edit_profile_info::put_update_profile_info))
        .route("/editor/update/profile-info/", put(edit_profile_info::put_update_profile_info))
        .route("/editor/update/profile-info", post(edit_profile_info::put_update_profile_info))
        .route("/editor/update/profile-info/", post(edit_profile_info::put_update_profile_info))

        .route("/editor/update/profile-password", get(edit_profile_password::get_edit_profile_password))
        .route("/editor/update/profile-password/", get(edit_profile_password::get_edit_profile_password))
        .route("/editor/update/profile-password", put(edit_profile_password::put_update_profile_password))
        .route("/editor/update/profile-password/", put(edit_profile_password::put_update_profile_password))
        .route("/editor/update/profile-password", post(edit_profile_password::put_update_profile_password))
        .route("/editor/update/profile-password/", post(edit_profile_password::put_update_profile_password))

        .route("/editor/update/profile-picture", get(edit_profile_picture::get_edit_profile_picture))
        .route("/editor/update/profile-picture/", get(edit_profile_picture::get_edit_profile_picture))
        .route("/editor/update/profile-picture", put(edit_profile_picture::put_update_profile_picture).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/update/profile-picture/", put(edit_profile_picture::put_update_profile_picture).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/update/profile-picture", post(edit_profile_picture::put_update_profile_picture).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/update/profile-picture/", post(edit_profile_picture::put_update_profile_picture).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))

        .route("/editor/create/song-album", get(edit_album::get_edit_album))
        .route("/editor/create/song-album/", get(edit_album::get_edit_album))
        .route("/editor/create/song-album", post(edit_album::post_create_album).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/create/song-album/", post(edit_album::post_create_album).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/create/song-album/{band}", get(edit_album::get_edit_album))
        .route("/editor/create/song-album/{band}/", get(edit_album::get_edit_album))
        .route("/editor/delete/song-album/{band}/{album}", get(delete_album::get_delete_album))
        .route("/editor/delete/song-album/{band}/{album}/", get(delete_album::get_delete_album))
        .route("/editor/delete/song-album/{band}/{album}", delete(delete_album::delete_album))
        .route("/editor/delete/song-album/{band}/{album}/", delete(delete_album::delete_album))
        .route("/editor/delete/song-album/{band}/{album}", post(delete_album::delete_album))
        .route("/editor/delete/song-album/{band}/{album}/", post(delete_album::delete_album))
        .route("/editor/update/song-album/{band}/{album}", get(edit_album::get_edit_album))
        .route("/editor/update/song-album/{band}/{album}/", get(edit_album::get_edit_album))
        .route("/editor/update/song-album/{band}/{album}", put(edit_album::put_update_album).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/update/song-album/{band}/{album}/", put(edit_album::put_update_album).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/update/song-album/{band}/{album}", post(edit_album::put_update_album).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))
        .route("/editor/update/song-album/{band}/{album}/", post(edit_album::put_update_album).layer(DefaultBodyLimit::max(1024 * 1024 * 8)))

        .route("/editor/create/tabs", get(edit_tabs::get_edit_tabs))
        .route("/editor/create/tabs/", get(edit_tabs::get_edit_tabs))
        .route("/editor/create/tabs/{band}", get(edit_tabs::get_edit_tabs))
        .route("/editor/create/tabs/{band}/", get(edit_tabs::get_edit_tabs))
        .route("/editor/create/tabs/{band}/{song}", get(edit_tabs::get_edit_tabs))
        .route("/editor/create/tabs/{band}/{song}/", get(edit_tabs::get_edit_tabs))
        .route("/editor/create/tabs", post(edit_tabs::post_create_tabs))
        .route("/editor/create/tabs/", post(edit_tabs::post_create_tabs))
        .route("/editor/update/tabs/{band}/{song}/{tab_type}/{contributor}", get(edit_tabs::get_edit_tabs))
        .route("/editor/update/tabs/{band}/{song}/{tab_type}/{contributor}/", get(edit_tabs::get_edit_tabs))
        .route("/editor/update/tabs/{band}/{song}/{tab_type}/{contributor}", put(edit_tabs::put_update_tabs))
        .route("/editor/update/tabs/{band}/{song}/{tab_type}/{contributor}/", put(edit_tabs::put_update_tabs))
        .route("/editor/update/tabs/{band}/{song}/{tab_type}/{contributor}", post(edit_tabs::put_update_tabs))
        .route("/editor/update/tabs/{band}/{song}/{tab_type}/{contributor}/", post(edit_tabs::put_update_tabs))
        .route("/editor/delete/tabs/{band}/{song}/{tab_type}/{contributor}", get(delete_tabs::get_delete_tabs))
        .route("/editor/delete/tabs/{band}/{song}/{tab_type}/{contributor}/", get(delete_tabs::get_delete_tabs))
        .route("/editor/delete/tabs/{band}/{song}/{tab_type}/{contributor}", delete(delete_tabs::delete_tabs))
        .route("/editor/delete/tabs/{band}/{song}/{tab_type}/{contributor}/", delete(delete_tabs::delete_tabs))
        .route("/editor/delete/tabs/{band}/{song}/{tab_type}/{contributor}", post(delete_tabs::delete_tabs))
        .route("/editor/delete/tabs/{band}/{song}/{tab_type}/{contributor}/", post(delete_tabs::delete_tabs))

        .route("/editor/create/video", get(edit_video::get_edit_video))
        .route("/editor/create/video/", get(edit_video::get_edit_video))
        .route("/editor/create/video", post(edit_video::post_create_video))
        .route("/editor/create/video/", post(edit_video::post_create_video))
        .route("/editor/create/video/{category}", get(edit_video::get_edit_video))
        .route("/editor/create/video/{category}/", get(edit_video::get_edit_video))
        .route("/editor/delete/video/{category}/{video}", get(delete_video::get_delete_video))
        .route("/editor/delete/video/{category}/{video}/", get(delete_video::get_delete_video))
        .route("/editor/delete/video/{category}/{video}", delete(delete_video::delete_video))
        .route("/editor/delete/video/{category}/{video}/", delete(delete_video::delete_video))
        .route("/editor/delete/video/{category}/{video}", post(delete_video::delete_video))
        .route("/editor/delete/video/{category}/{video}/", post(delete_video::delete_video))
        .route("/editor/update/video/{category}/{video}", get(edit_video::get_edit_video))
        .route("/editor/update/video/{category}/{video}/", get(edit_video::get_edit_video))
        .route("/editor/update/video/{category}/{video}", put(edit_video::put_update_video))
        .route("/editor/update/video/{category}/{video}/", put(edit_video::put_update_video))
        .route("/editor/update/video/{category}/{video}", post(edit_video::put_update_video))
        .route("/editor/update/video/{category}/{video}/", post(edit_video::put_update_video))

        .route("/editor/create/video-category", get(edit_video_category::get_edit_video_category))
        .route("/editor/create/video-category/", get(edit_video_category::get_edit_video_category))
        .route("/editor/create/video-category", post(edit_video_category::post_create_video_category))
        .route("/editor/create/video-category/", post(edit_video_category::post_create_video_category))
        .route("/editor/delete/video-category/{category}", get(delete_video_category::get_delete_video_category))
        .route("/editor/delete/video-category/{category}/", get(delete_video_category::get_delete_video_category))
        .route("/editor/delete/video-category/{category}", delete(delete_video_category::delete_video_category))
        .route("/editor/delete/video-category/{category}/", delete(delete_video_category::delete_video_category))
        .route("/editor/delete/video-category/{category}", post(delete_video_category::delete_video_category))
        .route("/editor/delete/video-category/{category}/", post(delete_video_category::delete_video_category))
        .route("/editor/update/video-category/{category}", get(edit_video_category::get_edit_video_category))
        .route("/editor/update/video-category/{category}/", get(edit_video_category::get_edit_video_category))
        .route("/editor/update/video-category/{category}", put(edit_video_category::put_update_video_category))
        .route("/editor/update/video-category/{category}/", put(edit_video_category::put_update_video_category))
        .route("/editor/update/video-category/{category}", post(edit_video_category::put_update_video_category))
        .route("/editor/update/video-category/{category}/", post(edit_video_category::put_update_video_category))

        .route("/forgot-password", get(forgot_password::get_forgot_password))
        .route("/forgot-password/", get(forgot_password::get_forgot_password))
        .route("/forgot-password", post(forgot_password::post_forgot_password))
        .route("/forgot-password/", post(forgot_password::post_forgot_password))
        .route("/password-reset/{session}", get(forgot_password::get_forgot_password))
        .route("/password-reset/{session}/", get(forgot_password::get_forgot_password))
        .route("/password-reset/{session}", post(forgot_password::post_reset_password))
        .route("/password-reset/{session}/", post(forgot_password::post_reset_password))

        .route("/lyrics", get(lyrics::get_lyrics))
        .route("/lyrics/", get(lyrics::get_lyrics))
        .route("/lyrics/{band}", get(lyrics::get_lyrics))
        .route("/lyrics/{band}/", get(lyrics::get_lyrics))
        .route("/lyrics/{band}/{album}", get(lyrics::get_lyrics))
        .route("/lyrics/{band}/{album}/", get(lyrics::get_lyrics))
        .route("/lyrics/{band}/{album}/{song}", get(lyrics::get_lyrics))
        .route("/lyrics/{band}/{album}/{song}/", get(lyrics::get_lyrics))
        .route("/lyrics.php", get(lyrics::get_lyrics_redirect))
        .route("/lyrics-booklet/{band}/{album}", get(lyrics_booklet::get_lyrics_booklet))
        .route("/lyrics-booklet/{band}/{album}/", get(lyrics_booklet::get_lyrics_booklet))

        .route("/members", get(members::get_members))
        .route("/members/", get(members::get_members))
        .route("/members/{username}", get(members::get_members))
        .route("/members/{username}/", get(members::get_members))
        .route("/members.php", get(members::get_members_redirect))

        .route("/photos", get(photos::get_photos))
        .route("/photos/", get(photos::get_photos))
        .route("/photos/{album}", get(photos::get_photos))
        .route("/photos/{album}/", get(photos::get_photos))
        .route("/photos/{album}/{photo}", get(photos::get_photos))
        .route("/photos/{album}/{photo}/", get(photos::get_photos))
        .route("/photos.php", get(photos::get_photos_redirect))

        .route("/privacy-policy", get(privacy_policy::get_privacy_policy))
        .route("/privacy-policy/", get(privacy_policy::get_privacy_policy))
        .route("/privacyPolicy.php", get(privacy_policy::get_privacy_policy_redirect))

        .route("/sign-in", get(sign_in::get_sign_in))
        .route("/sign-in/", get(sign_in::get_sign_in))
        .route("/sign-in", post(sign_in::post_sign_in))
        .route("/sign-in/", post(sign_in::post_sign_in))
        .route("/sign-out", post(sign_out::post_sign_out))
        .route("/sign-out/", post(sign_out::post_sign_out))
        .route("/sign-up", get(sign_up::get_sign_up))
        .route("/sign-up/", get(sign_up::get_sign_up))
        .route("/sign-up", post(sign_up::post_sign_up))
        .route("/sign-up/", post(sign_up::post_sign_up))
        .route("/signup.php", get(sign_up::get_sign_up_redirect))

        .route("/sitemap.xml", get(sitemap::get_sitemap)) // TODO - cache this response.

        .route("/tabs", get(tabs::get_tabs))
        .route("/tabs/", get(tabs::get_tabs))
        .route("/tabs/{band}", get(tabs::get_tabs))
        .route("/tabs/{band}/", get(tabs::get_tabs))
        .route("/tabs/{band}/{song}", get(tabs::get_tabs))
        .route("/tabs/{band}/{song}/", get(tabs::get_tabs))
        .route("/tabs/{band}/{song}/{tab_type}/{contributor}", get(tabs::get_tabs))
        .route("/tabs/{band}/{song}/{tab_type}/{contributor}/", get(tabs::get_tabs))
        .route("/tabs.php", get(tabs::get_tabs_redirect))

        .route("/terms-of-service", get(terms_of_service::get_terms_of_service))
        .route("/terms-of-service/", get(terms_of_service::get_terms_of_service))
        .route("/termsOfService.php", get(terms_of_service::get_terms_of_service_redirect))

        .route("/videos", get(videos::get_videos))
        .route("/videos/", get(videos::get_videos))
        .route("/videos/{category}", get(videos::get_videos))
        .route("/videos/{category}/", get(videos::get_videos))
        .route("/videos/{category}/{video}", get(videos::get_videos))
        .route("/videos/{category}/{video}/", get(videos::get_videos))
        .route("/videos.php", get(videos::get_videos_redirect))

        .layer(auth_layer)
        .fallback_service(get_service(ServeDir::new(uploaded_files_path)));
    
    app
}
