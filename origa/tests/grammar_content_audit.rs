//! Grammar content audit: no Korean (Hangul) artifacts (#178 W-11).
//!
//! During LLM-assisted grammar generation, Korean words were occasionally
//! emitted into Japanese example sentences (likely Korean was used as a pivot
//! language). This test guarantees that no rule in cdn/grammar/grammar.json
//! or cdn/grammar/rules/*.json carries any Hangul (U+AC00–U+D7AF) character.
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
            if let Some(idx) = s
                .chars()
                .position(|c| ('\u{AC00}'..='\u{D7AF}').contains(&c))
            {
                let context_start = idx.saturating_sub(40);
                let context_end = (idx + 40).min(s.len());
                hits.push(format!(
                    "{} = {:?}",
                    path.join("."),
                    &s[context_start..context_end]
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

#[test]
fn grammar_json_has_no_korean_artifacts() {
    let path = cdn_dir().join("grammar").join("grammar.json");
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
