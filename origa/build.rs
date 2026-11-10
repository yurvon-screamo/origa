//! Build-time corpus probe.
//!
//! The local CDN corpus is machine state outside the repository
//! (`$ORIGA_CDN_DIR` or `~/origa-cdn` — the resolution contract lives in
//! `src/corpus.rs`, which is the single source of truth). When the corpus
//! is not seeded, corpus-dependent tests skip green — and libtest captures
//! the in-test skip markers, so a default `cargo test` run would look
//! silently green. This probe surfaces the state where cargo can see it:
//! a `cargo:warning` printed on every build while the corpus is absent.
//! `manifest.json` is the sentinel every seeded corpus carries (the CDN
//! freshness manifest) — once the corpus appears, the rerun trigger
//! rebuilds this script and the warning stops.

use std::path::PathBuf;

fn main() {
    let corpus_dir = std::env::var_os("ORIGA_CDN_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("origa-cdn"));
            home.join("origa-cdn")
        });

    println!("cargo:rerun-if-env-changed=ORIGA_CDN_DIR");
    println!(
        "cargo:rerun-if-changed={}",
        corpus_dir.join("manifest.json").display()
    );

    // Sentinel per the module docs: manifest.json ships with every seeded
    // corpus. An existing-but-empty directory would otherwise be a silent
    // green (no warning, no markers) — exactly the state the probe exists
    // to surface.
    if !corpus_dir.join("manifest.json").exists() {
        println!(
            "cargo:warning=local corpus not seeded at {} (manifest.json missing) — \
             corpus-dependent tests will SKIP. Seed it once (see AGENTS.md -> \"Local corpus\")",
            corpus_dir.display()
        );
    }
}
