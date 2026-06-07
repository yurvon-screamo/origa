fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rustc-env=ORIGA_LANDING_BASE_URL={}",
        std::env::var("ORIGA_LANDING_BASE_URL").unwrap_or_else(|_| "https://origa.app".to_string())
    );

    let app_base_url = std::env::var("ORIGA_APP_BASE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            let base_uri = std::env::var("ORIGA_BASE_URI").ok();
            let app_prefix = std::env::var("ORIGA_APP_URI_PREFIX").ok();
            match (app_prefix, base_uri) {
                (Some(prefix), Some(base)) => format!("https://{prefix}.{base}"),
                _ => panic!(
                    "ORIGA_APP_BASE_URL or (ORIGA_APP_URI_PREFIX + ORIGA_BASE_URI) env vars must be set"
                ),
            }
        });
    println!("cargo:rustc-env=ORIGA_APP_BASE_URL={app_base_url}");
}
