//! Polysemic-kanji description audit (#178 W-10).
//!
//! Guards against LLM mistranslations where a polysemic English gloss produced a
//! Russian translation in the wrong sense. The canonical case is 字 (English
//! "character") being rendered as Russian "характер" (personality) instead of
//! the letter/symbol/sign sense the kanji actually carries.
//!
//! For each audited kanji we require at least one description_ru entry to
//! contain `required_ru_substring`. This catches both full regressions (the
//! fix gets reverted to a single wrong gloss) and partial regressions (the
//! fix is dropped, leaving the original mistranslation as the only entry).
//!
//! Optionally, `forbidden_as_sole_entry` forbids a specific gloss from being
//! the ONLY entry — used for cases where the previous gloss was entirely the
//! wrong sense (e.g. 封 = "тюлень" the animal). When the previous gloss was
//! merely too narrow but still valid (e.g. 房 = "комната" room, narrow but
//! correct), `forbidden_as_sole_entry` is None.
//!
//! The `cdn/` directory is gitignored. On a fresh clone without the kanji
//! store this test **gracefully skips** (passes with a stderr note) rather
//! than panic, so `cargo test --workspace` stays green in CI environments
//! that do not have the CDN artifacts. Once the store is present (local dev,
//! release CI after `scripts/deploy_cdn.py` has been run with the W-10
//! fixes), the audit runs for real.
//!
//! Run: `cargo test -p origa --test kanji_descriptions_audit`.

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
struct KanjiFile {
    kanji: Vec<KanjiEntry>,
}

#[derive(Deserialize)]
struct KanjiEntry {
    kanji: String,
    #[serde(default)]
    description_ru: Vec<String>,
}

struct AuditCase {
    kanji: &'static str,
    required_ru_substring: &'static str,
    /// If set, this gloss must NOT appear as a standalone single entry.
    /// None means the gloss is a valid (if narrow) sense and may coexist with
    /// the broader fix.
    forbidden_as_sole_entry: Option<&'static str>,
}

// Curated list mirroring scripts/fix_polysemic_kanji.py::POLYSEMIC_FIXES.
// Add new entries here whenever a new polysemic mistranslation is corrected.
const AUDIT_CASES: &[AuditCase] = &[
    AuditCase {
        kanji: "字",
        required_ru_substring: "знак",
        forbidden_as_sole_entry: Some("характер"),
    },
    AuditCase {
        kanji: "点",
        required_ru_substring: "точка",
        forbidden_as_sole_entry: Some("место"),
    },
    AuditCase {
        kanji: "州",
        required_ru_substring: "штат",
        forbidden_as_sole_entry: Some("состояние"),
    },
    AuditCase {
        kanji: "津",
        required_ru_substring: "переправа",
        forbidden_as_sole_entry: Some("входное отверстие"),
    },
    AuditCase {
        kanji: "軽",
        required_ru_substring: "лёгкий",
        forbidden_as_sole_entry: Some("свет"),
    },
    AuditCase {
        kanji: "封",
        required_ru_substring: "печать",
        forbidden_as_sole_entry: Some("тюлень"),
    },
    AuditCase {
        kanji: "箋",
        required_ru_substring: "записка",
        forbidden_as_sole_entry: Some("композиция"),
    },
    AuditCase {
        kanji: "底",
        required_ru_substring: "дно",
        forbidden_as_sole_entry: Some("нижний"),
    },
    AuditCase {
        kanji: "幹",
        required_ru_substring: "ствол",
        forbidden_as_sole_entry: Some("основная часть"),
    },
    AuditCase {
        kanji: "丁",
        required_ru_substring: "квартал",
        forbidden_as_sole_entry: Some("блокировать"),
    },
    AuditCase {
        kanji: "胞",
        required_ru_substring: "клетка",
        forbidden_as_sole_entry: Some("плацента"),
    },
    AuditCase {
        kanji: "衷",
        required_ru_substring: "чувства",
        forbidden_as_sole_entry: Some("сокровенный"),
    },
    AuditCase {
        kanji: "玩",
        required_ru_substring: "играть",
        forbidden_as_sole_entry: Some("игрушка"),
    },
    AuditCase {
        kanji: "彩",
        required_ru_substring: "цвет",
        forbidden_as_sole_entry: Some("раскрасить"),
    },
    AuditCase {
        kanji: "植",
        required_ru_substring: "сажать",
        forbidden_as_sole_entry: None,
    },
    AuditCase {
        kanji: "申",
        required_ru_substring: "вежливо",
        forbidden_as_sole_entry: None,
    },
    AuditCase {
        kanji: "述",
        required_ru_substring: "излагать",
        forbidden_as_sole_entry: None,
    },
    AuditCase {
        kanji: "候",
        required_ru_substring: "сезон",
        forbidden_as_sole_entry: None,
    },
    AuditCase {
        kanji: "兆",
        required_ru_substring: "предзнаменование",
        forbidden_as_sole_entry: None,
    },
    AuditCase {
        kanji: "符",
        required_ru_substring: "талисман",
        forbidden_as_sole_entry: None,
    },
    AuditCase {
        kanji: "房",
        required_ru_substring: "покой",
        forbidden_as_sole_entry: None,
    },
];

fn load_kanji_file() -> Option<KanjiFile> {
    let path = cdn_dir().join("dictionary").join("kanji.json");
    if !path.exists() {
        eprintln!(
            "[skip] kanji_descriptions_audit: {} is absent (cdn/ gitignored on fresh clones)",
            path.display()
        );
        return None;
    }
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let parsed: KanjiFile = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
    Some(parsed)
}

#[test]
fn polysemic_kanji_descriptions_are_not_mistranslated() {
    let Some(file) = load_kanji_file() else {
        return;
    };
    let mut problems: Vec<String> = Vec::new();

    for case in AUDIT_CASES {
        let Some(entry) = file.kanji.iter().find(|k| k.kanji == case.kanji) else {
            problems.push(format!(
                "{}: kanji entry missing from kanji.json",
                case.kanji
            ));
            continue;
        };

        let has_required = entry
            .description_ru
            .iter()
            .any(|d| d.contains(case.required_ru_substring));
        if !has_required {
            problems.push(format!(
                "{}: required substring {:?} missing from description_ru = {:?}",
                case.kanji, case.required_ru_substring, entry.description_ru
            ));
        }

        if let Some(forbidden) = case.forbidden_as_sole_entry {
            let is_sole_entry = entry.description_ru.len() == 1
                && entry.description_ru.first().map(|s| s.as_str()) == Some(forbidden);
            if is_sole_entry {
                problems.push(format!(
                    "{}: forbidden gloss {:?} is the sole description_ru entry — polysemy lost",
                    case.kanji, forbidden,
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "polysemic-kanji audit failed ({} problems):\n{}",
        problems.len(),
        problems.join("\n")
    );
}
