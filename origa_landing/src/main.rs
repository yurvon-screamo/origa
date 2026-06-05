#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use origa_landing::app::{App, shell};
    use tower_http::services::{ServeDir, ServeFile};

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

    let pkg_dir = env!("CARGO_MANIFEST_DIR");

    let conf = get_configuration(Some(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")))
        .expect("failed to read Leptos configuration from Cargo.toml");
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .nest_service("/images", ServeDir::new(format!("{pkg_dir}/public/images")))
        .route_service(
            "/landing.processed.css",
            ServeFile::new(format!("{pkg_dir}/style/landing.processed.css")),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback_service(
            ServeDir::new(format!("{pkg_dir}/public")).append_index_html_on_directories(false),
        )
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap_or_else(|e| panic!("server error: {e}"));
}
