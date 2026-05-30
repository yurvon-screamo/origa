use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use origa::dictionary::grammar::GrammarRule;
use origa::domain::{OrigaError, detect_grammar_rules_in_text, tokenize_text};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::dictionary::load_dictionary;

const PROGRESS_INTERVAL: usize = 5_000;

#[derive(Deserialize)]
struct PhraseIndexFile {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "h")]
    _hash: String,
    #[serde(rename = "phrases")]
    phrases: Vec<PhraseIndexEntry>,
}

#[derive(Deserialize)]
struct PhraseIndexEntry {
    #[serde(rename = "i")]
    id: String,
    #[serde(rename = "t")]
    tokens: Vec<String>,
    #[serde(rename = "c")]
    chunk_id: u32,
}

#[derive(Deserialize)]
struct PhraseChunkEntry {
    #[serde(rename = "i")]
    id: String,
    #[serde(rename = "x")]
    text: String,
}

#[derive(Serialize)]
struct EnrichedIndexFile {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "h")]
    hash: String,
    #[serde(rename = "total")]
    total: usize,
    #[serde(rename = "phrases")]
    phrases: Vec<EnrichedEntry>,
}

#[derive(Clone, Serialize)]
struct EnrichedEntry {
    #[serde(rename = "i")]
    id: String,
    #[serde(rename = "t")]
    tokens: Vec<String>,
    #[serde(rename = "c")]
    chunk_id: u32,
    #[serde(rename = "g")]
    grammar_rules: Vec<String>,
}

fn load_grammar_rules(grammar_path: &PathBuf) -> Result<Vec<GrammarRule>, OrigaError> {
    let content = fs::read_to_string(grammar_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read grammar.json: {}", e),
    })?;
    let stripped = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    #[derive(Deserialize)]
    struct GrammarStore {
        grammar: Vec<GrammarRule>,
    }

    let store: GrammarStore =
        serde_json::from_str(stripped).map_err(|e| OrigaError::GrammarParseError {
            reason: format!("Failed to parse grammar.json: {}", e),
        })?;

    Ok(store.grammar)
}

fn load_phrase_index(path: &PathBuf) -> Result<PhraseIndexFile, OrigaError> {
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read phrase index: {}", e),
    })?;
    serde_json::from_str(&content).map_err(|e| OrigaError::PhraseParseError {
        reason: format!("Failed to parse phrase index: {}", e),
    })
}

fn compute_hash(entries: &[EnrichedEntry]) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));

    let serializable: Vec<serde_json::Value> = sorted
        .iter()
        .map(|e| {
            serde_json::json!({
                "i": e.id,
                "t": e.tokens,
                "c": e.chunk_id,
                "g": e.grammar_rules,
            })
        })
        .collect();

    let serialized = serde_json::to_string(&serializable).expect("enriched entries must serialize");

    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn run_enrich_phrases_with_grammar(
    input: PathBuf,
    chunks_dir: PathBuf,
    grammar: PathBuf,
    output: PathBuf,
    dictionary_dir: Option<PathBuf>,
) -> Result<(), OrigaError> {
    tracing::info!("Loading dictionary for tokenizer...");
    if let Some(ref dict_dir) = dictionary_dir {
        if dict_dir.exists() {
            tracing::info!("Using dictionary dir: {}", dict_dir.display());
        }
    }
    load_dictionary()?;

    tracing::info!("Loading grammar rules from {}...", grammar.display());
    let rules = load_grammar_rules(&grammar)?;
    tracing::info!("Loaded {} grammar rules", rules.len());

    tracing::info!("Loading phrase index from {}...", input.display());
    let index = load_phrase_index(&input)?;
    let total = index.phrases.len();
    tracing::info!("Loaded {} phrase entries", total);

    let mut enriched_entries: Vec<EnrichedEntry> = Vec::with_capacity(total);
    let mut chunk_cache: HashMap<u32, Vec<PhraseChunkEntry>> = HashMap::new();

    for (idx, entry) in index.phrases.iter().enumerate() {
        let chunk_entries = chunk_cache
            .entry(entry.chunk_id)
            .or_insert_with(|| load_chunk_entries(&chunks_dir, entry.chunk_id));

        let text = match chunk_entries.iter().find(|e| e.id == entry.id) {
            Some(e) => &e.text,
            None => {
                tracing::warn!("Phrase {} not found in chunk {}", entry.id, entry.chunk_id);
                continue;
            },
        };

        let tokens = tokenize_text(text).unwrap_or_default();
        let grammar_ids = detect_grammar_rules_in_text(text, &tokens, &rules);
        let grammar_strs: Vec<String> = grammar_ids.iter().map(|id| id.to_string()).collect();

        enriched_entries.push(EnrichedEntry {
            id: entry.id.clone(),
            tokens: entry.tokens.clone(),
            chunk_id: entry.chunk_id,
            grammar_rules: grammar_strs,
        });

        if (idx + 1) % PROGRESS_INTERVAL == 0 {
            let pct = (idx + 1) as f64 / total as f64 * 100.0;
            tracing::info!("[{}/{}] {:.1}% enriched", idx + 1, total, pct);
        }
    }

    let hash = compute_hash(&enriched_entries);
    let new_version = index.version + 1;

    let output_file = EnrichedIndexFile {
        version: new_version,
        hash,
        total: enriched_entries.len(),
        phrases: enriched_entries,
    };

    let json = serde_json::to_string(&output_file).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to serialize enriched index: {}", e),
    })?;
    fs::write(&output, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", output.display(), e),
    })?;

    tracing::info!("Enriched index written to {}", output.display());
    tracing::info!(
        "Version: {} -> {}, {} phrases, hash: {}...{}",
        index.version,
        new_version,
        output_file.total,
        &output_file.hash[..16],
        &output_file.hash[output_file.hash.len() - 8..]
    );

    Ok(())
}

fn load_chunk_entries(chunks_dir: &Path, chunk_id: u32) -> Vec<PhraseChunkEntry> {
    let chunk_path = chunks_dir.join(format!("p{}.json", chunk_id));
    let content = match fs::read_to_string(&chunk_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to read chunk {}: {}", chunk_path.display(), e);
            return vec![];
        },
    };
    match serde_json::from_str(&content) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Failed to parse chunk {}: {}", chunk_path.display(), e);
            vec![]
        },
    }
}
