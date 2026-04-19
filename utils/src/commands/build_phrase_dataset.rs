use crate::dictionary::load_dictionary;
use crate::utils::{collect_json_files, get_base_path};
use origa::domain::{OrigaError, tokenize_text};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const PROGRESS_INTERVAL: usize = 10_000;
const MAX_RANKED_MISSING: usize = 200;

#[derive(Deserialize)]
struct Transcription {
    id: u64,
    text: String,
    audio_ref: String,
}

#[derive(Serialize)]
struct PhraseEntry {
    id: u64,
    text: String,
    audio_ref: String,
    tokens: Vec<String>,
    token_count: usize,
}

#[derive(Serialize)]
struct DatasetStats {
    total: usize,
    passed_filter: usize,
    skipped_too_short: usize,
    skipped_no_vocab_coverage: usize,
    unique_tokens_in_phrases: usize,
    avg_tokens_per_phrase: f64,
}

#[derive(Serialize)]
struct PhraseDatasetOutput {
    stats: DatasetStats,
    phrases: Vec<PhraseEntry>,
}

#[derive(Serialize)]
struct GapsSummary {
    total_phrases_checked: usize,
    phrases_fully_covered: usize,
    phrases_partial_covered: usize,
    phrases_zero_coverage: usize,
    unique_missing_words: usize,
    vocab_coverage_pct: f64,
    phrases_lost_by_missing_count: HashMap<String, usize>,
}

#[derive(Serialize, Clone)]
struct MissingWordEntry {
    word: String,
    frequency: u32,
    would_unlock_phrases: u32,
    example_phrase: String,
}

#[derive(Serialize)]
struct VocabGapsOutput {
    summary: GapsSummary,
    missing_words_ranked: Vec<MissingWordEntry>,
}

struct PhraseAnalysis {
    vocab_bases: Vec<String>,
    missing_bases: Vec<String>,
}

fn load_vocabulary_dictionary(base_path: &Path) -> Result<HashSet<String>, OrigaError> {
    let mut words = HashSet::new();
    let vocab_path = base_path
        .join("origa_ui")
        .join("public")
        .join("dictionary")
        .join("vocabulary");

    if !vocab_path.exists() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Vocabulary directory not found: {}", vocab_path.display()),
        });
    }

    let files = collect_json_files(&vocab_path)?;
    for file in files {
        let content = fs::read_to_string(&file).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", file.display(), e),
        })?;
        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to parse {}: {}", file.display(), e),
            })?;
        if let Some(obj) = json.as_object() {
            for key in obj.keys() {
                words.insert(key.clone());
            }
        }
    }

    if words.is_empty() {
        return Err(OrigaError::TokenizerError {
            reason: format!("No vocabulary words loaded from {}", vocab_path.display()),
        });
    }
    tracing::info!("Loaded {} words from vocabulary dictionary", words.len());
    Ok(words)
}

fn load_transcriptions(path: &Path) -> Result<Vec<Transcription>, OrigaError> {
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;
    serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to parse transcriptions: {}", e),
    })
}

fn analyze_phrase(text: &str, vocabulary: &HashSet<String>) -> Result<PhraseAnalysis, OrigaError> {
    let tokens = tokenize_text(text)?;
    let vocab_bases: Vec<String> = tokens
        .iter()
        .filter(|t| t.part_of_speech().is_vocabulary_word())
        .map(|t| t.orthographic_base_form().to_string())
        .collect();

    let missing_bases: Vec<String> = vocab_bases
        .iter()
        .filter(|b| !vocabulary.contains(*b))
        .cloned()
        .collect();

    Ok(PhraseAnalysis {
        vocab_bases,
        missing_bases,
    })
}

fn missing_count_bucket(count: usize) -> String {
    match count {
        1 => "1_missing".to_string(),
        2 => "2_missing".to_string(),
        3 => "3_missing".to_string(),
        _ => "4_plus_missing".to_string(),
    }
}

fn build_missing_words_ranked(
    word_block_count: &HashMap<String, u32>,
    word_unlock_count: &HashMap<String, u32>,
    word_example: &HashMap<String, String>,
) -> Vec<MissingWordEntry> {
    let mut entries: Vec<MissingWordEntry> = word_block_count
        .iter()
        .map(|(word, &freq)| MissingWordEntry {
            word: word.clone(),
            frequency: freq,
            would_unlock_phrases: word_unlock_count.get(word).copied().unwrap_or(0),
            example_phrase: word_example.get(word).cloned().unwrap_or_default(),
        })
        .collect();

    entries.sort_by(|a, b| b.frequency.cmp(&a.frequency));
    entries.truncate(MAX_RANKED_MISSING);
    entries
}

fn write_json_file(path: &Path, data: &impl Serialize) -> Result<(), OrigaError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to create directory {}: {}", parent.display(), e),
        })?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to serialize JSON: {}", e),
    })?;
    fs::write(path, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", path.display(), e),
    })
}

fn print_progress(
    processed: usize,
    total: usize,
    passed: usize,
    too_short: usize,
    no_coverage: usize,
) {
    let pct = (processed as f64 / total as f64) * 100.0;
    tracing::info!(
        "[{}/{}] {:.1}% — {} passed, {} skipped (too short), {} skipped (no vocab)",
        processed,
        total,
        pct,
        passed,
        too_short,
        no_coverage
    );
}

struct ProcessingSummary<'a> {
    total: usize,
    passed: usize,
    too_short: usize,
    no_coverage: usize,
    unique_tokens: usize,
    avg_tokens: f64,
    elapsed: std::time::Duration,
    top_missing: &'a [MissingWordEntry],
}

fn print_final_summary(summary: &ProcessingSummary) {
    let pct = |n: usize| (n as f64 / summary.total as f64) * 100.0;
    tracing::info!(
        "Done! Results (elapsed: {:.1}s):",
        summary.elapsed.as_secs_f64()
    );
    tracing::info!("  Total:            {}", summary.total);
    tracing::info!(
        "  Passed filter:     {} ({:.1}%)",
        summary.passed,
        pct(summary.passed)
    );
    tracing::info!(
        "  Too short:         {} ({:.1}%)",
        summary.too_short,
        pct(summary.too_short)
    );
    tracing::info!(
        "  No vocab coverage: {} ({:.1}%)",
        summary.no_coverage,
        pct(summary.no_coverage)
    );
    tracing::info!("  Unique tokens in phrases: {}", summary.unique_tokens);
    tracing::info!("  Avg tokens per phrase: {:.1}", summary.avg_tokens);

    let top_n = std::cmp::min(10, summary.top_missing.len());
    if top_n > 0 {
        tracing::info!("Top {} missing words:", top_n);
        for entry in &summary.top_missing[..top_n] {
            tracing::info!(
                "  {:10} — {} phrases blocked (would unlock ~{})",
                entry.word,
                entry.frequency,
                entry.would_unlock_phrases
            );
        }
    }
}

fn categorize_coverage(analysis: &PhraseAnalysis) -> CoverageCategory {
    let has_any = !analysis.vocab_bases.is_empty();
    let all_missing = analysis.vocab_bases.len() == analysis.missing_bases.len();
    let no_missing = analysis.missing_bases.is_empty();

    if !has_any || all_missing {
        CoverageCategory::Zero
    } else if no_missing {
        CoverageCategory::Full
    } else {
        CoverageCategory::Partial
    }
}

enum CoverageCategory {
    Full,
    Partial,
    Zero,
}

pub fn run_build_phrase_dataset(
    input: PathBuf,
    output: PathBuf,
    min_tokens: usize,
) -> Result<(), OrigaError> {
    let start = Instant::now();

    tracing::info!("Loading tokenizer...");
    load_dictionary()?;

    let mut base_path = get_base_path();
    if base_path.ends_with("utils") {
        base_path.pop();
    }

    tracing::info!("Loading vocabulary...");
    let vocabulary = load_vocabulary_dictionary(&base_path)?;
    tracing::info!("Loaded {} vocabulary words", vocabulary.len());

    tracing::info!("Loading transcriptions from {}...", input.display());
    let transcriptions = load_transcriptions(&input)?;
    let total = transcriptions.len();
    tracing::info!("Processing {} transcriptions...", total);

    let mut passed_phrases: Vec<PhraseEntry> = Vec::new();
    let mut skipped_too_short = 0usize;
    let mut skipped_no_coverage = 0usize;
    let mut fully_covered = 0usize;
    let mut partial_covered = 0usize;
    let mut zero_covered = 0usize;
    let mut total_tokens_in_passed = 0usize;
    let mut all_unique_tokens: HashSet<String> = HashSet::new();
    let mut all_unique_missing: HashSet<String> = HashSet::new();
    let mut all_unique_vocab: HashSet<String> = HashSet::new();
    let mut all_unique_covered: HashSet<String> = HashSet::new();
    let mut word_block_count: HashMap<String, u32> = HashMap::new();
    let mut word_unlock_count: HashMap<String, u32> = HashMap::new();
    let mut word_example: HashMap<String, String> = HashMap::new();
    let mut lost_by_missing: HashMap<String, usize> = HashMap::new();

    for (idx, tr) in transcriptions.iter().enumerate() {
        let analysis = analyze_phrase(&tr.text, &vocabulary)?;

        for base in &analysis.vocab_bases {
            all_unique_vocab.insert(base.clone());
            if vocabulary.contains(base) {
                all_unique_covered.insert(base.clone());
            } else {
                all_unique_missing.insert(base.clone());
            }
        }

        match categorize_coverage(&analysis) {
            CoverageCategory::Full => fully_covered += 1,
            CoverageCategory::Partial => partial_covered += 1,
            CoverageCategory::Zero => zero_covered += 1,
        }

        let has_missing = !analysis.missing_bases.is_empty();

        if analysis.vocab_bases.len() < min_tokens {
            skipped_too_short += 1;
        } else if has_missing {
            skipped_no_coverage += 1;
            track_gaps(
                &analysis,
                &tr.text,
                &mut word_block_count,
                &mut word_unlock_count,
                &mut word_example,
                &mut lost_by_missing,
            );
        } else {
            let token_count = analysis.vocab_bases.len();
            for base in &analysis.vocab_bases {
                all_unique_tokens.insert(base.clone());
            }
            total_tokens_in_passed += token_count;
            passed_phrases.push(PhraseEntry {
                id: tr.id,
                text: tr.text.clone(),
                audio_ref: tr.audio_ref.clone(),
                tokens: analysis.vocab_bases,
                token_count,
            });
        }

        if (idx + 1) % PROGRESS_INTERVAL == 0 {
            print_progress(
                idx + 1,
                total,
                passed_phrases.len(),
                skipped_too_short,
                skipped_no_coverage,
            );
        }
    }

    let avg_tokens = if passed_phrases.is_empty() {
        0.0
    } else {
        total_tokens_in_passed as f64 / passed_phrases.len() as f64
    };
    let vocab_coverage_pct = if all_unique_vocab.is_empty() {
        0.0
    } else {
        (all_unique_covered.len() as f64 / all_unique_vocab.len() as f64) * 100.0
    };

    let ranked = build_missing_words_ranked(&word_block_count, &word_unlock_count, &word_example);

    let dataset_output = PhraseDatasetOutput {
        stats: DatasetStats {
            total,
            passed_filter: passed_phrases.len(),
            skipped_too_short,
            skipped_no_vocab_coverage: skipped_no_coverage,
            unique_tokens_in_phrases: all_unique_tokens.len(),
            avg_tokens_per_phrase: (avg_tokens * 10.0).round() / 10.0,
        },
        phrases: passed_phrases,
    };

    let gaps_output = VocabGapsOutput {
        summary: GapsSummary {
            total_phrases_checked: total,
            phrases_fully_covered: fully_covered,
            phrases_partial_covered: partial_covered,
            phrases_zero_coverage: zero_covered,
            unique_missing_words: all_unique_missing.len(),
            vocab_coverage_pct: (vocab_coverage_pct * 10.0).round() / 10.0,
            phrases_lost_by_missing_count: lost_by_missing,
        },
        missing_words_ranked: ranked.clone(),
    };

    let dataset_path = output.join("phrase_dataset.json");
    let gaps_path = output.join("vocab_gaps.json");
    write_json_file(&dataset_path, &dataset_output)?;
    write_json_file(&gaps_path, &gaps_output)?;

    print_final_summary(&ProcessingSummary {
        total,
        passed: dataset_output.stats.passed_filter,
        too_short: skipped_too_short,
        no_coverage: skipped_no_coverage,
        unique_tokens: all_unique_tokens.len(),
        avg_tokens,
        elapsed: start.elapsed(),
        top_missing: &ranked,
    });

    tracing::info!("Results saved to:");
    tracing::info!(
        "  {} ({} phrases)",
        dataset_path.display(),
        dataset_output.stats.passed_filter
    );
    tracing::info!("  {} (coverage analysis)", gaps_path.display());

    Ok(())
}

fn track_gaps(
    analysis: &PhraseAnalysis,
    phrase_text: &str,
    word_block_count: &mut HashMap<String, u32>,
    word_unlock_count: &mut HashMap<String, u32>,
    word_example: &mut HashMap<String, String>,
    lost_by_missing: &mut HashMap<String, usize>,
) {
    let bucket = missing_count_bucket(analysis.missing_bases.len());
    *lost_by_missing.entry(bucket).or_insert(0) += 1;

    for word in &analysis.missing_bases {
        *word_block_count.entry(word.clone()).or_insert(0) += 1;
        word_example
            .entry(word.clone())
            .or_insert_with(|| phrase_text.to_string());
    }

    if analysis.missing_bases.len() == 1 {
        *word_unlock_count
            .entry(analysis.missing_bases[0].clone())
            .or_insert(0) += 1;
    }
}
