//! Well-known sets JLPT level audit (#178 S-3).
//!
//! Guards against the blanket N5 tag previously applied to every Duolingo and
//! Spy x Family set regardless of actual content difficulty. The Duolingo
//! Section number encoded in each title (1-6) maps deterministically to a JLPT
//! level (Section 1-2 → N5, 3-4 → N4, 5-6 → N3); Spy x Family content files
//! all carry `level: "N3"` in their own metadata.
//!
//! Run: `cargo test -p origa --test well_known_sets_audit`.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

fn cdn_dir() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root is parent of CARGO_MANIFEST_DIR")
        .join("cdn")
}

#[derive(Deserialize)]
struct SetMeta {
    id: String,
    set_type: String,
    level: String,
    title_en: Option<String>,
    title_ru: Option<String>,
}

fn load_meta() -> Vec<SetMeta> {
    let path = cdn_dir()
        .join("well_known_set")
        .join("well_known_sets_meta.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
}

fn section_from_title(title: &str) -> Option<u32> {
    let mut iter = title.split_whitespace();
    while let Some(word) = iter.next() {
        let lower = word.to_lowercase();
        if lower == "section" || lower == "модуль" {
            if let Some(num_word) = iter.next() {
                if let Ok(num) = num_word.parse::<u32>() {
                    return Some(num);
                }
            }
        }
    }
    None
}

fn expected_duolingo_level(title: &str) -> Option<&'static str> {
    match section_from_title(title)? {
        1 | 2 => Some("N5"),
        3 | 4 => Some("N4"),
        5 | 6 => Some("N3"),
        _ => None,
    }
}

#[test]
fn duolingo_sets_match_section_to_level_mapping() {
    let records = load_meta();
    let mut problems: Vec<String> = Vec::new();
    let mut checked = 0u32;

    for record in records.iter() {
        if record.set_type != "DuolingoEn" && record.set_type != "DuolingoRu" {
            continue;
        }
        let title_blob = format!(
            "{} {}",
            record.title_en.as_deref().unwrap_or(""),
            record.title_ru.as_deref().unwrap_or("")
        );
        let Some(expected) = expected_duolingo_level(&title_blob) else {
            problems.push(format!(
                "[{}] Duolingo set with no Section/Модуль in title: {:?}",
                record.id, title_blob
            ));
            continue;
        };
        checked += 1;
        if record.level != expected {
            problems.push(format!(
                "[{}] level={:?} but title section implies {:?}: {:?}",
                record.id, record.level, expected, title_blob
            ));
        }
    }

    assert!(
        !problems.is_empty() || checked > 0,
        "audit did not check any Duolingo sets — fixture path or set_type values drifted"
    );
    assert!(
        problems.is_empty(),
        "Duolingo level audit failed ({} problems out of {} checked):\n{}",
        problems.len(),
        checked,
        problems.join("\n")
    );
}

#[test]
fn spy_family_sets_are_n3() {
    let records = load_meta();
    let spy_records: Vec<&SetMeta> = records
        .iter()
        .filter(|r| r.set_type == "SpyFamily")
        .collect();

    assert!(
        !spy_records.is_empty(),
        "no SpyFamily sets found in well_known_sets_meta.json — fixture path drifted"
    );

    let wrong: Vec<&SetMeta> = spy_records.iter().filter(|r| r.level != "N3").copied().collect();
    assert!(
        wrong.is_empty(),
        "Spy x Family sets must all be tagged N3 (matches content file level); these are wrong:\n{}",
        wrong
            .iter()
            .map(|r| format!("  [{}] level={}", r.id, r.level))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
