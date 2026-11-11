//! Drift guards for the web Docker font staging (see `origa_ui/Dockerfile`).
//!
//! The web production image stages brand fonts from the prod CDN before
//! `trunk build`: the woff2 subset list comes from the FONTS block of
//! `end2end/cdn-manifest.txt` (sed-extracted in the Dockerfile), and the
//! download base falls back to `DEFAULT_CDN` from `build_defaults.rs` when
//! `ORIGA_CDN_BASE_URL` is unset. All three sources live outside the Rust
//! compilation unit, so a drift between them compiles silently — and a stale
//! fallback or an emptied manifest degrades web typography to system
//! fallbacks again (the exact bug fixed by PR #928). These tests fail first
//! and point at the drift instead of letting it ship unnoticed. The full
//! build-time behaviour (download, staging, dist guard) is asserted by the
//! post-build guard inside the Dockerfile itself and exercised by the CI
//! `Docker Build (ui)` job — this file pins the cross-file consistency that
//! no single build can catch.

use std::collections::HashSet;

const DOCKERFILE: &str = include_str!("../Dockerfile");
const BUILD_RS: &str = include_str!("../build.rs");
const BUILD_DEFAULTS: &str = include_str!("../../build_defaults.rs");
const CDN_MANIFEST: &str = include_str!("../../end2end/cdn-manifest.txt");

/// Filenames listed in the FONTS block of the CDN manifest — the canonical
/// subset list shared by the Docker build download step, the download-cdn
/// CI action and the font_face.rs manifest guard.
fn manifest_font_names() -> Vec<String> {
    let block = CDN_MANIFEST
        .split("# FONTS-START")
        .nth(1)
        .and_then(|rest| rest.split("# FONTS-END").next())
        .expect("cdn-manifest.txt must contain a # FONTS-START / # FONTS-END block");
    block
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// The `DEFAULT_CDN` value parsed out of `build_defaults.rs` (the constant
/// lives in a build-script compilation unit and is not importable here).
fn default_cdn() -> String {
    let needle = "pub(crate) const DEFAULT_CDN: &str = ";
    let rest = BUILD_DEFAULTS
        .lines()
        .find_map(|line| line.trim().strip_prefix(needle))
        .expect("build_defaults.rs must define DEFAULT_CDN");
    rest.trim()
        .trim_end_matches(';')
        .trim()
        .trim_matches('"')
        .to_string()
}

/// The fallback URL of the Dockerfile download step
/// (`${ORIGA_CDN_BASE_URL:-<fallback>}`), used when the build runs without
/// an explicit `ORIGA_CDN_BASE_URL` (local builds; CI always passes one).
fn dockerfile_cdn_fallback() -> String {
    let needle = "${ORIGA_CDN_BASE_URL:-";
    let rest = DOCKERFILE
        .lines()
        .find_map(|line| line.trim().split_once(needle))
        .map(|(_, tail)| tail)
        .expect(
            "origa_ui/Dockerfile must embed a ${ORIGA_CDN_BASE_URL:-<fallback>} \
             download base",
        );
    rest.split('}')
        .next()
        .expect("closing brace checked above")
        .trim()
        .to_string()
}

/// The Dockerfile download base and the build.rs CDN fallback must stay the
/// same host: `build.rs` bakes `resolve_cdn()`'s URL into the wasm bundle,
/// so a drifted fallback would download fonts from one host while the built
/// app fetches them from another — silent content mismatch.
#[test]
fn dockerfile_cdn_fallback_matches_default_cdn() {
    let expected = default_cdn();
    let actual = dockerfile_cdn_fallback();
    assert_eq!(
        actual, expected,
        "the Dockerfile download fallback drifted from DEFAULT_CDN \
         (build_defaults.rs): local no-env builds would stage fonts from \
         {actual} while the wasm bundle fetches from {expected} — keep the \
         two in sync (the fallback mirrors resolve_cdn())"
    );
}

/// The corpus-root contract between the Dockerfile and `build.rs` (corpus
/// migration #912): build scripts resolve the corpus root via the
/// `ORIGA_CDN_DIR` override (falling back to `$HOME/origa-cdn`), and the
/// Dockerfile pins that override to the directory it stages the subsets
/// into. A drift between the two sides is invisible at compile time and
/// only surfaces as a red Docker build guard deep in CI (this exact bug
/// shipped as the round-1 failure of the font-staging task) — the pin
/// fails in every Test run instead, right after either side renames its
/// half of the contract.
#[test]
fn dockerfile_staging_root_follows_build_rs_corpus_contract() {
    // build.rs side of the contract: cdn_dir() reads the ORIGA_CDN_DIR
    // override and stages fonts from <corpus root>/fonts.
    for (needle, what) in [
        (
            "env::var_os(\"ORIGA_CDN_DIR\")",
            "the ORIGA_CDN_DIR override of cdn_dir()",
        ),
        (
            ".join(\"fonts\")",
            "the fonts subdirectory of the corpus root",
        ),
    ] {
        assert!(
            BUILD_RS.contains(needle),
            "origa_ui/build.rs no longer contains {what} ({needle:?}) — \
             the corpus-root contract drifted from the Dockerfile staging \
             (see ENV ORIGA_CDN_DIR in origa_ui/Dockerfile); keep both \
             sides of the contract in sync, otherwise the Docker build \
             stages fonts the build script never reads"
        );
    }

    // Dockerfile side: the override is set and the download step targets
    // it, so the staged root and the build script's resolution cannot
    // silently diverge.
    assert!(
        DOCKERFILE.contains("ENV ORIGA_CDN_DIR="),
        "origa_ui/Dockerfile no longer pins ORIGA_CDN_DIR — the build \
         script would fall back to the machine-local corpus root \
         (~/origa-cdn), which no Docker builder seeds, and FONT_FILES \
         would compile empty again (the PR #928 bug)"
    );
    assert!(
        DOCKERFILE.contains("${ORIGA_CDN_DIR}/fonts"),
        "the Dockerfile download step no longer stages into \
         ${{ORIGA_CDN_DIR}}/fonts — the staging root drifted from the \
         cdn_dir() override it must feed"
    );
}

/// The FONTS block is the download input: emptied or malformed, the Docker
/// build loses its source of truth. The in-image post-build guard catches
/// this only at image-build time; this pin catches it in every Test run
/// where the manifest path itself is in the PR's diff scope.
#[test]
fn fonts_block_lists_served_woff2_subsets() {
    let names = manifest_font_names();
    assert!(
        !names.is_empty(),
        "the FONTS block of cdn-manifest.txt is empty — the Docker build \
         would stage no fonts and @font-face would degrade to system \
         fallbacks (the PR #928 bug)"
    );
    let mut seen = HashSet::new();
    for name in &names {
        assert!(
            !name.contains('#'),
            "manifest comment leaked into the font list: {name:?}"
        );
        let path = name.split('/').collect::<Vec<_>>();
        assert_eq!(
            path.len(),
            2,
            "font entry must be `fonts/<filename>.woff2`, got {name:?}"
        );
        assert_eq!(
            path[0], "fonts",
            "font entry must live under fonts/, got {name:?}"
        );
        let file = path[1];
        let stem = file
            .strip_suffix(".woff2")
            .unwrap_or_else(|| panic!("font entry must be a .woff2 subset, got {name:?}"));
        assert!(!stem.is_empty(), "empty font stem in {name:?}");
        assert!(
            stem.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'),
            "unexpected characters in font filename {name:?} (content-hashed \
             subsets are [A-Za-z0-9._-]+)"
        );
        assert!(seen.insert(name), "duplicate font entry: {name:?}");
    }
}
