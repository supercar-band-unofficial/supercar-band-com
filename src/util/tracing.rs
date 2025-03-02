use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("axum=info,supercar_band_com=debug"))
        .init();
}
