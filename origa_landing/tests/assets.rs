//! Asset integrity of the landing's static images.
//!
//! Two guarantees, both about the same failure mode: an image in the
//! repository that nothing uses, or an image the code asks for that is not
//! there.
//!
//! 1. The literal asset paths the pages reference resolve to a file under
//!    `public/`. The list is hand-maintained: a new literal reference needs a
//!    new entry, and a missing file fails here rather than in production.
//! 2. The set of files under `public/images/` is exactly the allowlist: no
//!    orphan ships to production, no referenced file goes missing. The
//!    dynamic families (`{lang}.{name}.webp`, `{lang}.car.{slide}.webp`)
//!    cannot be recovered from the source by scanning it — the allowlist is
//!    what covers them.
//!
//! TRIPWIRE, NOT TECH DEBT: adding a landing image means adding one line to
//! `EXPECTED_IMAGES`. Deleting the check instead is how a 10 MB pile of
//! unused PNGs comes back (ADR-068).
//!
//! Pure module (no leptos/ssr deps) — builds with empty default features,
//! like `build_config.rs` and `css_rebuild.rs`.

use std::collections::BTreeSet;
use std::path::Path;

const PUBLIC_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/public");
const IMAGES_DIR: &str = "images";

/// Every file allowed under `public/images/`.
const EXPECTED_IMAGES: [&str; 26] = [
    // Collages and the logo — referenced literally from the pages and from
    // the root READMEs (which render them through relative paths).
    "images/content.png",
    "images/en.all_in_one.png",
    "images/en.hero.png",
    "images/en.learn.png",
    "images/logo.png",
    "images/ru.all_in_one.png",
    "images/ru.hero.png",
    "images/ru.learn.png",
    // Application captures — webp, two locales, built from format strings.
    "images/app/en.car.grammar.webp",
    "images/app/en.car.lesson.webp",
    "images/app/en.car.main.webp",
    "images/app/en.car.phrases.webp",
    "images/app/en.car.writing.webp",
    "images/app/en.grammar.webp",
    "images/app/en.phrase.webp",
    "images/app/en.writing.webp",
    "images/app/feat-art.webp",
    "images/app/hero-art.webp",
    "images/app/ru.car.grammar.webp",
    "images/app/ru.car.lesson.webp",
    "images/app/ru.car.main.webp",
    "images/app/ru.car.phrases.webp",
    "images/app/ru.car.writing.webp",
    "images/app/ru.grammar.webp",
    "images/app/ru.phrase.webp",
    "images/app/ru.writing.webp",
];

/// Paths the pages reference literally. Hand-maintained: the compiler cannot
/// extract them from `format!` strings, and a Leptos view is not introspected
/// at test time.
const REFERENCED_PATHS: [&str; 6] = [
    "images/logo.png",          // components/layout.rs — the header logo
    "images/content.png",       // pages/integrations.rs — the hero image
    "images/app/feat-art.webp", // pages/features.rs
    "images/app/hero-art.webp", // pages/home.rs
    "og-image.png",             // components/seo.rs — the JSON-LD image
    "favicon.png",              // the site icon
];

fn actual_images() -> BTreeSet<String> {
    let root = Path::new(PUBLIC_DIR).join(IMAGES_DIR);
    let mut found = BTreeSet::new();
    let stack = vec![root.clone()];
    let mut todo = stack;
    while let Some(dir) = todo.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                todo.push(path);
            } else if let Ok(relative) = path.strip_prefix(&root) {
                found.insert(
                    Path::new(IMAGES_DIR)
                        .join(relative)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    found
}

fn expected_images() -> BTreeSet<String> {
    EXPECTED_IMAGES
        .iter()
        .map(|path| path.to_string())
        .collect()
}

#[test]
fn every_referenced_asset_exists() {
    for path in REFERENCED_PATHS {
        let full = Path::new(PUBLIC_DIR).join(path);
        assert!(
            full.is_file(),
            "the pages reference {path} but {path} is not under public/",
        );
    }
}

#[test]
fn no_orphan_images_ship_to_production() {
    let actual = actual_images();
    let expected = expected_images();
    let orphans: Vec<_> = actual.difference(&expected).collect();

    assert!(
        orphans.is_empty(),
        "public/images/ holds files no page references (ADR-068): {orphans:#?}\n\
         either reference them, or remove them, or add them to EXPECTED_IMAGES",
    );
}

#[test]
fn every_expected_image_is_present() {
    let actual = actual_images();
    let expected = expected_images();
    let missing: Vec<_> = expected.difference(&actual).collect();

    assert!(
        missing.is_empty(),
        "the pages reference images that are not in public/images/: {missing:#?}",
    );
}
