#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "origa_landing=info,leptos_router=warn".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Server running on {addr}");

    let leptos_options = match leptos::config::get_configuration(Some(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/Cargo.toml"
    ))) {
        Ok(conf) => {
            tracing::info!("Loaded Leptos config from Cargo.toml");
            conf.leptos_options
        },
        Err(e) => {
            tracing::warn!("Failed to read Cargo.toml config ({e}), using defaults");
            leptos::config::LeptosOptions::builder()
                .output_name("origa_landing")
                .build()
        },
    };

    let app = origa_landing::server::build_router(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap_or_else(|e| panic!("server error: {e}"));
}
