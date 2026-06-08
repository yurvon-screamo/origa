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

    build_css();
}

fn build_css() {
    println!("cargo:rerun-if-changed=style/landing.css");
    println!("cargo:rerun-if-changed=style/input.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let input = std::path::Path::new(&manifest_dir).join("style/landing.css");
    let output = std::path::Path::new(&manifest_dir).join("style/landing.processed.css");

    let skip = output.exists()
        && input
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .is_some_and(|input_mtime| {
                output
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .is_some_and(|output_mtime| output_mtime >= input_mtime)
            });

    if skip {
        return;
    }

    let result = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args([
                "/C",
                "npx",
                "tailwindcss",
                "--input",
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--minify",
            ])
            .current_dir(&manifest_dir)
            .status()
    } else {
        std::process::Command::new("npx")
            .args([
                "tailwindcss",
                "--input",
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--minify",
            ])
            .current_dir(&manifest_dir)
            .status()
    };

    match result {
        Ok(s) if s.success() => {},
        Ok(s) => panic!("CSS build failed: npx tailwindcss exited with {s}"),
        Err(e) => {
            println!("cargo:warning=npx not available, skipping CSS rebuild: {e}");
        },
    }
}
