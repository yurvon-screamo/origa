//! Offline builder for the grammar tokenization precompute blob (#521).
//!
//! Walks every string of `grammar_v2.json` + `grammar_ko_vi.json` and
//! derives precompute entries for the exact strings the grammar pages
//! render:
//!
//! - markdown fields (`explanation`, `how_to_form`, `examples`,
//!   `pro_tip`, `warnings[]`) are rendered through the same
//!   markdown→html→text-node pipeline `MarkdownText` uses at runtime, and
//!   every kanji-bearing text node becomes a store key;
//! - every other kanji-bearing string (`title`, `nuances` mistake pairs,
//!   notes, related pattern titles) becomes a whole-string key — the
//!   `FuriganaText` path.
//!
//! One deterministic blob is written to `grammar/grammar_precompute.rkyv`.
//! The pipeline below (pulldown-cmark options, ammonia defaults, scraper
//! traversal with the ruby/rt/rp skip) must stay in lockstep with
//! `origa_ui/src/ui_components/markdown.rs` — the fixed-expectation tests
//! pin the node extraction for a realistic grammar fixture.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ego_tree::NodeRef;
use origa::dictionary::cdn_blob::{self, BlobHeader, SCHEMA_VERSION, build_blob, split_blob};
use origa::dictionary::precompute_blob::{
    PrecomputeBlob, deflate_blob, inflate_blob, serialize_precompute_blob_to_rkyv,
};
use origa::domain::{JapaneseText, OrigaError};
use scraper::{Html, Node};

use super::precompute_common::{
    init_furigana_and_dictionary_ingredients, precompute_entry, sha256_hex, sha256_raw,
};
use crate::dictionary::load_dictionary;

const GRAMMAR_SOURCE: &str = "grammar/grammar_v2.json";
const GRAMMAR_OVERLAY_SOURCE: &str = "grammar/grammar_ko_vi.json";
const FURIGANA_SOURCE: &str = "dictionaries/JmdictFurigana.txt";
const GRAMMAR_PRECOMPUTE_BLOB: &str = "grammar/grammar_precompute.rkyv";

/// Fields the grammar pages render through `MarkdownText`. Everything
/// else reaches the screen as a plain string (often via `FuriganaText`).
const MARKDOWN_FIELDS: &[&str] = &[
    "explanation",
    "how_to_form",
    "examples",
    "pro_tip",
    "warnings",
];

/// Same skip set as `markdown.rs`: furigana is never nested into existing
/// ruby markup.
const SKIP_TAGS: &[&str] = &["ruby", "rt", "rp"];

/// The exact markdown→html rendering `MarkdownText` performs before
/// walking text nodes.
fn render_markdown(content: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    ammonia::clean(&html_output)
}

/// Kanji-bearing text nodes of the rendered markdown, de-duplicated and
/// sorted — these are the store keys for the `MarkdownText` path.
pub fn furiganizable_markdown_nodes(markdown: &str) -> Vec<String> {
    let document = Html::parse_document(&render_markdown(markdown));
    let mut nodes = collect_text_nodes(document.tree.root(), false);
    nodes.sort();
    nodes.dedup();
    nodes
}

fn collect_text_nodes(node_ref: NodeRef<'_, Node>, in_skip: bool) -> Vec<String> {
    let mut result = Vec::new();
    match node_ref.value() {
        Node::Text(text) => {
            let text_str: &str = text;
            if !in_skip && text_str.contains_kanji() {
                result.push(text_str.to_string());
            }
        },
        Node::Element(elem) => {
            let should_skip = in_skip || SKIP_TAGS.contains(&elem.name());
            for child in node_ref.children() {
                result.extend(collect_text_nodes(child, should_skip));
            }
        },
        _ => {
            for child in node_ref.children() {
                result.extend(collect_text_nodes(child, in_skip));
            }
        },
    }
    result
}

/// Collects the precompute keys of one JSON value: markdown fields get
/// their text nodes, every other kanji-bearing string is keyed whole.
fn collect_value_keys(value: &serde_json::Value, key: &str, keys: &mut BTreeMap<String, ()>) {
    match value {
        serde_json::Value::Object(map) => {
            for (child_key, child) in map {
                collect_value_keys(child, child_key, keys);
            }
        },
        serde_json::Value::Array(items) => {
            for item in items {
                collect_value_keys(item, key, keys);
            }
        },
        serde_json::Value::String(text) => {
            if MARKDOWN_FIELDS.contains(&key) {
                for node in furiganizable_markdown_nodes(text) {
                    keys.insert(node, ());
                }
            } else if text.contains_kanji() {
                keys.insert(text.clone(), ());
            }
        },
        _ => {},
    }
}

pub fn run_build_grammar_precompute(cdn_dir: Option<&Path>) -> Result<(), OrigaError> {
    let cdn_dir = cdn_dir.unwrap_or_else(|| Path::new("cdn"));
    load_dictionary().map_err(|e| OrigaError::RepositoryError {
        reason: format!("tokenizer dictionary unavailable for precompute: {e:?}"),
    })?;

    let dictionary_ingredients =
        init_furigana_and_dictionary_ingredients(&cdn_dir.join(FURIGANA_SOURCE))?;

    let grammar_v2 =
        fs::read(cdn_dir.join(GRAMMAR_SOURCE)).map_err(|e| OrigaError::RepositoryError {
            reason: format!("failed to read {GRAMMAR_SOURCE}: {e}"),
        })?;
    let overlay = fs::read(cdn_dir.join(GRAMMAR_OVERLAY_SOURCE)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("failed to read {GRAMMAR_OVERLAY_SOURCE}: {e}"),
        }
    })?;

    // Freshness binds to both grammar sources plus the tokenizer inputs —
    // a dictionary bump regenerates the blob even though the grammar
    // files are unchanged.
    let mut source_with_ingredients = Vec::new();
    source_with_ingredients.extend_from_slice(&grammar_v2);
    source_with_ingredients.extend_from_slice(&overlay);
    source_with_ingredients.extend_from_slice(&dictionary_ingredients);

    let existing = fs::read(cdn_dir.join(GRAMMAR_PRECOMPUTE_BLOB))
        .ok()
        .and_then(|deflated| inflate_blob(&deflated).ok());
    let blob_is_fresh = matches!(existing.as_deref().map(split_blob), Some(Ok((header, _))) if
            header.schema_version == SCHEMA_VERSION
                && header.source_sha256 == sha256_raw(&source_with_ingredients));
    if blob_is_fresh {
        tracing::info!("{GRAMMAR_PRECOMPUTE_BLOB} is fresh, skipping regeneration");
        return Ok(());
    }

    let mut keys = BTreeMap::new();
    for source in [&grammar_v2, &overlay] {
        let value: serde_json::Value =
            serde_json::from_slice(source).map_err(|e| OrigaError::RepositoryError {
                reason: format!("failed to parse grammar source: {e}"),
            })?;
        collect_value_keys(&value, "", &mut keys);
    }

    let started = std::time::Instant::now();
    let mut entries = BTreeMap::new();
    for key in keys.into_keys() {
        if let Some(entry) = precompute_entry(&key) {
            entries.insert(key, entry);
        }
    }
    tracing::info!(
        "grammar precompute: {} entries derived in {:.1}s",
        entries.len(),
        started.elapsed().as_secs_f32()
    );

    let payload = serialize_precompute_blob_to_rkyv(&PrecomputeBlob { entries }).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("failed to serialize {GRAMMAR_PRECOMPUTE_BLOB}: {e:?}"),
        }
    })?;

    // Guard covers the manifest-visible sources (both grammar JSONs) in
    // path order — clients with a fetched manifest can verify it.
    let guard = cdn_blob::manifest_guard_from_hex_hashes(&[
        &sha256_hex(&grammar_v2),
        &sha256_hex(&overlay),
    ]);
    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256: sha256_raw(&source_with_ingredients),
        manifest_guard: guard,
    };
    let built = build_blob(&header, &payload);
    fs::write(cdn_dir.join(GRAMMAR_PRECOMPUTE_BLOB), deflate_blob(&built)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("failed to write {GRAMMAR_PRECOMPUTE_BLOB}: {e}"),
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixed-expectation parity fixture: a realistic grammar explanation
    /// with inline code, a code fence carrying Japanese, and a table. The
    /// extracted nodes must be exactly the kanji-bearing text fragments
    /// the runtime `add_furigana_to_html` would visit: a code fence is a
    /// single text node (Japanese line + Latin line together), inline
    /// kana-only code is filtered out by the kanji gate.
    #[test]
    fn markdown_nodes_match_runtime_text_fragments() {
        let markdown = "Pattern `～は～です` is **basic**.\n\n\
```\n私は学生です。\nI am a student.\n```\n\n\
| Element | Function |\n|---------|----------|\n| [話] は | topic |\n";

        let nodes = furiganizable_markdown_nodes(markdown);

        assert_eq!(
            nodes,
            vec![
                "[話] は".to_string(),
                "私は学生です。\nI am a student.\n".to_string(),
            ]
        );
    }

    #[test]
    fn markdown_nodes_skip_ruby_markup() {
        // Ammonia strips the raw <ruby> markup a source might carry; the
        // surviving kanji-bearing fragments become the keys.
        let markdown = "既存 <ruby>食<rt>た</rt></ruby>べる と 日本語";

        let nodes = furiganizable_markdown_nodes(markdown);

        assert_eq!(
            nodes,
            vec!["べる と 日本語".to_string(), "既存 ".to_string()]
        );
    }

    #[test]
    fn value_keys_split_markdown_fields_and_plain_strings() {
        let value = serde_json::json!({
            "title": "～は～です",
            "short_description": "no kanji here",
            "nuances": {
                "common_mistakes": [
                    {"wrong": "学生 です", "correct": "学生です", "note": null}
                ]
            },
            "examples": "```\n田中さんは先生です。\n```"
        });

        let mut keys = BTreeMap::new();
        collect_value_keys(&value, "", &mut keys);

        let sorted: Vec<String> = keys.into_keys().collect();
        assert_eq!(
            sorted,
            vec![
                "学生 です".to_string(),
                "学生です".to_string(),
                "田中さんは先生です。\n".to_string(),
            ]
        );
    }
}
