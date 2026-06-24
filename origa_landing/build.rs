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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    build_css();
    generate_sitemap(&manifest_dir);
}

fn build_css() {
    println!("cargo:rerun-if-changed=style/landing.css");
    println!("cargo:rerun-if-changed=style/input.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let base = std::path::Path::new(&manifest_dir);
    let input = base.join("style/landing.css");
    let input_extra = base.join("style/input.css");
    let output = base.join("style/landing.processed.css");

    let output_mtime = output.metadata().ok().and_then(|m| m.modified().ok());

    let skip = output.exists()
        && output_mtime.is_some_and(|out_time| {
            let main_fresh = input
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .is_some_and(|t| out_time >= t);

            let extra_fresh = input_extra
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .is_some_and(|t| out_time >= t);

            main_fresh && extra_fresh
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

/// Render `public/sitemap.xml` from `public/sitemap.xml.tmpl`, substituting
/// `{{LASTMOD}}` with the most authoritative build date available.
///
/// Precedence: `ORIGA_BUILD_DATE` (set by CI, e.g. `docker.yml`) → last commit
/// date of the template (`git log`) → `1970-01-01` sentinel with a warning.
/// The sentinel keeps local builds green in environments without git history
/// (e.g. a fresh tarball checkout); CI always supplies `ORIGA_BUILD_DATE`.
fn generate_sitemap(manifest_dir: &str) {
    println!("cargo:rerun-if-changed=public/sitemap.xml.tmpl");
    println!("cargo:rerun-if-env-changed=ORIGA_BUILD_DATE");

    let base = std::path::Path::new(manifest_dir);
    let tmpl_path = base.join("public/sitemap.xml.tmpl");
    let out_path = base.join("public/sitemap.xml");

    let tmpl = match std::fs::read_to_string(&tmpl_path) {
        Ok(s) => s,
        Err(e) => panic!("failed to read {}: {e}", tmpl_path.display()),
    };

    let lastmod = std::env::var("ORIGA_BUILD_DATE")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| git_lastmod(manifest_dir))
        .unwrap_or_else(|| {
            println!(
                "cargo:warning=sitemap lastmod: no ORIGA_BUILD_DATE and no git, using 1970-01-01"
            );
            "1970-01-01".to_string()
        });

    let rendered = tmpl.replace("{{LASTMOD}}", &lastmod);

    std::fs::write(&out_path, rendered)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}

/// Best-effort last-modified date of the sitemap template, as an ISO-8601
/// `YYYY-MM-DD` string. Returns `None` when git is unavailable (no `.git`,
/// git not on PATH) so the caller can fall back to the sentinel.
///
/// `--follow` spans the `sitemap.xml -> sitemap.xml.tmpl` rename so a fresh
/// checkout still resolves a date from the template's pre-rename history.
fn git_lastmod(manifest_dir: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%cd",
            "--date=short",
            "--follow",
            "--",
            "public/sitemap.xml.tmpl",
        ])
        .current_dir(manifest_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if date.is_empty() { None } else { Some(date) }
}
