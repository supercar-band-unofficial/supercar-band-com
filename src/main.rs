use std::env;
use std::net::SocketAddr;

mod database;
mod router;
mod ui_modules;
mod ui_pages;
mod ui_primitives;
mod util;

#[tokio::main]
async fn main() {
    env::set_var("MEMORY_SERVE_QUIET", "1");

    let _ = util::tracing::init_tracing();
    let _ = database::init_pool().await;
    let _ = router::authn::init_user_sessions().await;
    let _ = util::captcha::init_captchas();
    let _ = util::password_reset_session::init_password_reset_sessions();
    let _ = util::rate_limit::init_rate_limits();
    let _ = util::smtp::init_mailer();
    tokio::spawn(util::image_upload::init_temporary_image_upload_cleanup());

    let app = router::initialize();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030")
        .await
        .unwrap();

    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await.unwrap();
}
