use super::kanji_common::{KanjiEntry, create_backup, read_dictionary, write_dictionary};
use origa::domain::OrigaError;
use std::collections::HashSet;
use std::mem;
use std::path::PathBuf;

fn dedup_preserve_order(vec: &mut Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut removed = Vec::new();
    let original = mem::take(vec);
    for item in original {
        if seen.insert(item.clone()) {
            vec.push(item);
        } else {
            removed.push(item);
        }
    }
    removed
}

struct DedupStats {
    total_kanji: usize,
    affected_kanji: usize,
    on_duplicates_removed: usize,
    kun_duplicates_removed: usize,
    details: Vec<KanjiDedupDetail>,
}

struct KanjiDedupDetail {
    kanji: String,
    on_removed: Vec<String>,
    kun_removed: Vec<String>,
}

fn process_entries(entries: &mut [KanjiEntry]) -> DedupStats {
    let total_kanji = entries.len();
    let mut affected_kanji = 0;
    let mut on_duplicates_removed = 0;
    let mut kun_duplicates_removed = 0;
    let mut details = Vec::new();

    for entry in entries.iter_mut() {
        let on_removed = dedup_preserve_order(&mut entry.on_readings);
        let kun_removed = dedup_preserve_order(&mut entry.kun_readings);

        if !on_removed.is_empty() || !kun_removed.is_empty() {
            affected_kanji += 1;
            on_duplicates_removed += on_removed.len();
            kun_duplicates_removed += kun_removed.len();

            details.push(KanjiDedupDetail {
                kanji: entry.kanji.clone(),
                on_removed,
                kun_removed,
            });
        }
    }

    DedupStats {
        total_kanji,
        affected_kanji,
        on_duplicates_removed,
        kun_duplicates_removed,
        details,
    }
}

fn log_stats(stats: &DedupStats) {
    tracing::info!("Total kanji processed: {}", stats.total_kanji);
    tracing::info!("Kanji affected: {}", stats.affected_kanji);
    tracing::info!(
        "On-readings duplicates removed: {}",
        stats.on_duplicates_removed
    );
    tracing::info!(
        "Kun-readings duplicates removed: {}",
        stats.kun_duplicates_removed
    );

    for detail in &stats.details {
        if !detail.on_removed.is_empty() {
            tracing::info!(
                "  {} — removed on duplicates: {:?}",
                detail.kanji,
                detail.on_removed
            );
        }
        if !detail.kun_removed.is_empty() {
            tracing::info!(
                "  {} — removed kun duplicates: {:?}",
                detail.kanji,
                detail.kun_removed
            );
        }
    }
}

pub fn run_dedup_kanji_readings(input: PathBuf, dry_run: bool) -> Result<(), OrigaError> {
    tracing::info!("Reading kanji dictionary: {}", input.display());
    let mut dictionary = read_dictionary(&input)?;

    let stats = process_entries(&mut dictionary.kanji);
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
