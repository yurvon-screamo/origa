//! Grammar content audit: no Korean (Hangul) artifacts (#178 W-11).
//!
//! During LLM-assisted grammar generation, Korean words were occasionally
//! emitted into Japanese example sentences (likely Korean was used as a pivot
//! language). This test guarantees that no rule in cdn/grammar/grammar.json
//! or cdn/grammar/rules/*.json carries any Hangul (U+AC00–U+D7AF) character.
//!
//! The `cdn/` directory is gitignored. On a fresh clone without the grammar
//! store the tests **gracefully skip** (pass with a stderr note) rather than
//! panic, so `cargo test --workspace` stays green in CI environments that
//! do not have the CDN artifacts. Once the store is present (local dev,
//! release CI after `scripts/deploy_cdn.py` has been run with the W-11
//! fixes), the audit runs for real.
//!
//! Run: `cargo test -p origa --test grammar_content_audit`.

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
struct GrammarFile {
    #[serde(default)]
    grammar: Vec<serde_json::Value>,
}

fn scan_for_hangul(value: &serde_json::Value, path: &mut Vec<String>, hits: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => {
            // Find the byte offset of the first Hangul char via char_indices,
            // then build a context window on a char boundary so slicing never
            // panics. Mixing char-position (from `.chars().position()`) with
            // byte slicing would crash on multi-byte scripts like Japanese or
            // Hangul itself.
            let first_hangul_byte = s
                .char_indices()
                .find(|(_, c)| ('\u{AC00}'..='\u{D7AF}').contains(c))
                .map(|(byte_idx, _)| byte_idx);
            if let Some(byte_idx) = first_hangul_byte {
                let window_start = prev_char_boundary(s, byte_idx.saturating_sub(120));
                let window_end = next_char_boundary(s, (byte_idx + 120).min(s.len()));
                hits.push(format!(
                    "{} = {:?}",
                    path.join("."),
                    &s[window_start..window_end]
                ));
            }
        },
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                path.push(k.clone());
                scan_for_hangul(v, path, hits);
                path.pop();
            }
        },
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                path.push(format!("[{i}]"));
                scan_for_hangul(v, path, hits);
                path.pop();
            }
        },
        _ => {},
    }
}

fn prev_char_boundary(s: &str, mut byte_idx: usize) -> usize {
    while byte_idx > 0 && !s.is_char_boundary(byte_idx) {
        byte_idx -= 1;
    }
    byte_idx
}

fn next_char_boundary(s: &str, mut byte_idx: usize) -> usize {
    while byte_idx < s.len() && !s.is_char_boundary(byte_idx) {
        byte_idx += 1;
    }
    byte_idx
}

#[test]
fn grammar_json_has_no_korean_artifacts() {
    let path = cdn_dir().join("grammar").join("grammar.json");
    if !path.exists() {
        eprintln!(
            "[skip] grammar_json_has_no_korean_artifacts: {} is absent (cdn/ gitignored on fresh clones)",
            path.display()
        );
        return;
    }
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let parsed: GrammarFile = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));

    let mut hits: Vec<String> = Vec::new();
    for rule in &parsed.grammar {
        let rule_id = rule
            .get("rule_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let mut path_stack = vec![format!("rule_id={rule_id}")];
        scan_for_hangul(rule, &mut path_stack, &mut hits);
    }

    assert!(
        hits.is_empty(),
        "grammar.json contains {} Korean (Hangul) fragment(s) — likely LLM artifact(s):\n{}",
        hits.len(),
        hits.join("\n")
    );
}

#[test]
fn grammar_rule_files_have_no_korean_artifacts() {
    let rules_dir = cdn_dir().join("grammar").join("rules");
    if !rules_dir.exists() {
        eprintln!(
            "[skip] grammar_rule_files_have_no_korean_artifacts: {} is absent (cdn/ gitignored on fresh clones)",
            rules_dir.display()
        );
        return;
    }
    let mut hits: Vec<String> = Vec::new();
    let mut files_scanned = 0u32;

    for entry in fs::read_dir(&rules_dir).expect("rules dir must exist") {
        let entry = entry.expect("dir entry must be readable");
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        for rule_entry in fs::read_dir(entry.path()).expect("subdir must be readable") {
            let rule_path = rule_entry.expect("rule entry must be readable").path();
            if rule_path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            files_scanned += 1;
            let raw = fs::read_to_string(&rule_path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", rule_path.display()));
            let parsed: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e}", rule_path.display()));
            let mut path_stack = vec![rule_path.display().to_string()];
            scan_for_hangul(&parsed, &mut path_stack, &mut hits);
        }
    }

    assert!(
        hits.is_empty(),
        "rules/* contains {} Korean (Hangul) fragment(s) across {files_scanned} files:\n{}",
        hits.len(),
        hits.join("\n")
    );
}
