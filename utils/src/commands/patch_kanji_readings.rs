use super::kanji_common::{KanjiEntry, create_backup, read_dictionary, write_dictionary};
use origa::domain::OrigaError;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Patch {
    kanji: String,
    #[serde(default)]
    remove_on: Vec<String>,
    #[serde(default)]
    remove_kun: Vec<String>,
}

struct PatchResult {
    on_removed: Vec<String>,
    kun_removed: Vec<String>,
}

struct PatchStats {
    total_patches: usize,
    applied_patches: usize,
    missing_kanji: Vec<String>,
    total_on_removed: usize,
    total_kun_removed: usize,
    details: Vec<(String, PatchResult)>,
}

fn read_patches(path: &PathBuf) -> Result<Vec<Patch>, OrigaError> {
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read patches {}: {}", path.display(), e),
    })?;
    serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to parse patches: {}", e),
    })
}

fn remove_readings(readings: &mut Vec<String>, to_remove: &HashSet<String>) -> Vec<String> {
    let original_len = readings.len();
    readings.retain(|r| !to_remove.contains(r));
    if readings.len() == original_len {
        return Vec::new();
    }
    to_remove
        .iter()
        .filter(|r| !readings.contains(*r))
        .cloned()
        .collect()
}

fn apply_patch(
    entry: &mut KanjiEntry,
    remove_on: &HashSet<String>,
    remove_kun: &HashSet<String>,
) -> PatchResult {
    let on_removed = remove_readings(&mut entry.on_readings, remove_on);
    let kun_removed = remove_readings(&mut entry.kun_readings, remove_kun);
    PatchResult {
        on_removed,
        kun_removed,
    }
}

fn build_kanji_index(entries: &[KanjiEntry]) -> HashMap<String, usize> {
    entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.kanji.clone(), i))
        .collect()
}

fn process_patches(entries: &mut [KanjiEntry], patches: &[Patch]) -> PatchStats {
    let index = build_kanji_index(entries);

    let mut stats = PatchStats {
        total_patches: patches.len(),
        applied_patches: 0,
        missing_kanji: Vec::new(),
        total_on_removed: 0,
        total_kun_removed: 0,
        details: Vec::new(),
    };

    for patch in patches {
        let Some(&idx) = index.get(&patch.kanji) else {
            stats.missing_kanji.push(patch.kanji.clone());
            tracing::warn!("Kanji not found in dictionary: {}", patch.kanji);
            continue;
        };

        let remove_on: HashSet<String> = patch.remove_on.iter().cloned().collect();
        let remove_kun: HashSet<String> = patch.remove_kun.iter().cloned().collect();

        let result = apply_patch(&mut entries[idx], &remove_on, &remove_kun);

        if !result.on_removed.is_empty() || !result.kun_removed.is_empty() {
            stats.applied_patches += 1;
            stats.total_on_removed += result.on_removed.len();
            stats.total_kun_removed += result.kun_removed.len();
            stats.details.push((patch.kanji.clone(), result));
        }
    }

    stats
}

fn log_stats(stats: &PatchStats) {
    tracing::info!("Patches loaded: {}", stats.total_patches);
    tracing::info!("Patches applied: {}", stats.applied_patches);
    tracing::info!("On-readings removed: {}", stats.total_on_removed);
    tracing::info!("Kun-readings removed: {}", stats.total_kun_removed);

    if !stats.missing_kanji.is_empty() {
        tracing::warn!("Kanji not found in dictionary: {:?}", stats.missing_kanji);
    }

    for (kanji, result) in &stats.details {
        if !result.on_removed.is_empty() {
            tracing::info!("  {} — removed on: {:?}", kanji, result.on_removed);
        }
        if !result.kun_removed.is_empty() {
            tracing::info!("  {} — removed kun: {:?}", kanji, result.kun_removed);
        }
    }
}

pub fn run_patch_kanji_readings(
    input: PathBuf,
    patches: PathBuf,
    dry_run: bool,
) -> Result<(), OrigaError> {
    tracing::info!("Reading patches: {}", patches.display());
    let patch_list = read_patches(&patches)?;

    tracing::info!("Reading kanji dictionary: {}", input.display());
    let mut dictionary = read_dictionary(&input)?;

    let stats = process_patches(&mut dictionary.kanji, &patch_list);
    log_stats(&stats);

    if dry_run {
        tracing::info!("Dry run — no changes written");
        return Ok(());
    }

    let backup_path = create_backup(&input)?;
    tracing::info!("Backup saved: {}", backup_path.display());

    write_dictionary(&input, &dictionary)?;
    tracing::info!("Kanji dictionary updated: {}", input.display());
    Ok(())
}
