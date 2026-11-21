use super::kanji_common::{create_backup, read_dictionary, write_dictionary};
use super::kanji_corpus::{KanjiCorpusIndex, is_kanji, katakana_to_hiragana};
use origa::dictionary::furigana_dict::FuriganaDictionary;
use origa::domain::OrigaError;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

struct CollapseDetail {
    kanji: String,
    on_removed: Vec<String>,
    kun_removed: Vec<String>,
}

pub fn run_collapse_kanji_readings(
    input: PathBuf,
    furigana_path: PathBuf,
    apply: bool,
) -> Result<(), OrigaError> {
    tracing::info!("Reading kanji dictionary: {}", input.display());
    let mut dictionary = read_dictionary(&input)?;

    tracing::info!("Reading furigana corpus: {}", furigana_path.display());
    let furigana_text =
        fs::read_to_string(&furigana_path).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", furigana_path.display(), e),
        })?;
    let furigana = FuriganaDictionary::from_text(&furigana_text)?;
    let corpus = KanjiCorpusIndex::build(&furigana);

    let mut details = Vec::new();
    for entry in &mut dictionary.kanji {
        let popular = entry.popular_words.clone();
        let (kun_kept, kun_removed) =
            collapse_kun_readings(&entry.kanji, &entry.kun_readings, &corpus, &popular);
        let (on_kept, on_removed) = collapse_on_readings(&entry.kanji, &entry.on_readings, &corpus);
        if kun_removed.is_empty() && on_removed.is_empty() {
            continue;
        }
        entry.kun_readings = kun_kept;
        entry.on_readings = on_kept;
        entry
            .reading_frequencies
            .retain(|reading, _| !kun_removed.contains(reading) && !on_removed.contains(reading));
        details.push(CollapseDetail {
            kanji: entry.kanji.clone(),
            on_removed,
            kun_removed,
        });
    }

    log_report(dictionary.kanji.len(), &details);

    if !apply {
        tracing::info!("Dry run — no changes written (pass --apply to write)");
        return Ok(());
    }

    let backup_path = create_backup(&input)?;
    tracing::info!("Backup saved: {}", backup_path.display());
    write_dictionary(&input, &dictionary)?;
    tracing::info!("Kanji dictionary updated: {}", input.display());
    Ok(())
}

/// Same-body kun readings collapse to one representative: the reading
/// matching a popular word of the kanji (first match in list order), else
/// the shortest form. Readings with an ambiguous or corpus-absent body
/// stay untouched.
fn collapse_kun_readings(
    kanji: &str,
    readings: &[String],
    corpus: &KanjiCorpusIndex,
    popular_words: &[String],
) -> (Vec<String>, Vec<String>) {
    let Some(kanji_char) = leading_kanji(kanji) else {
        return (readings.to_vec(), Vec::new());
    };
    let popular_readings = corpus.popular_full_readings(kanji_char, popular_words);

    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, reading) in readings.iter().enumerate() {
        let mut bodies = corpus.kun_body_candidates(kanji_char, &katakana_to_hiragana(reading));
        if bodies.len() == 1 {
            let body = bodies.remove(0);
            groups.entry(body).or_default().push(index);
        }
    }

    let mut removed: Vec<usize> = Vec::new();
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let representative = members
            .iter()
            .copied()
            .find(|&index| popular_readings.contains(&katakana_to_hiragana(&readings[index])))
            .unwrap_or_else(|| {
                *members
                    .iter()
                    .min_by_key(|&&index| (readings[index].chars().count(), index))
                    .expect("collapse group is never empty")
            });
        removed.extend(
            members
                .iter()
                .copied()
                .filter(|&index| index != representative),
        );
    }
    split_kept_and_removed(readings, removed)
}

/// Prefix and gemination variants of one on morpheme collapse to the
/// most corpus-frequent surface (shortest then list order on ties).
/// Go-on/kan-on doublets differing in the first mora (カン/ケン) are not
/// related by either rule and stay untouched.
fn collapse_on_readings(
    kanji: &str,
    readings: &[String],
    corpus: &KanjiCorpusIndex,
) -> (Vec<String>, Vec<String>) {
    let Some(kanji_char) = leading_kanji(kanji) else {
        return (readings.to_vec(), Vec::new());
    };
    let hiragana: Vec<String> = readings
        .iter()
        .map(|reading| katakana_to_hiragana(reading))
        .collect();
    let counts: Vec<usize> = hiragana
        .iter()
        .map(|reading| corpus.exact_reading_count(kanji_char, reading))
        .collect();

    let mut parent: Vec<usize> = (0..readings.len()).collect();
    for a in 0..readings.len() {
        for b in (a + 1)..readings.len() {
            if is_on_variant_pair(&hiragana[a], &hiragana[b]) {
                union(&mut parent, a, b);
            }
        }
    }

    let mut removed: Vec<usize> = Vec::new();
    for group in root_groups(&parent) {
        if group.len() < 2 {
            continue;
        }
        let mut ranked = group;
        ranked.sort_by_key(|&index| {
            (
                std::cmp::Reverse(counts[index]),
                hiragana[index].chars().count(),
                index,
            )
        });
        removed.extend(ranked.into_iter().skip(1));
    }
    split_kept_and_removed(readings, removed)
}

fn is_on_variant_pair(a: &str, b: &str) -> bool {
    if a.starts_with(b) || b.starts_with(a) {
        return true;
    }
    let geminated_a = strip_gemination(a);
    if geminated_a != a && (b.starts_with(&geminated_a) || geminated_a.starts_with(b)) {
        return true;
    }
    let geminated_b = strip_gemination(b);
    geminated_b != b && (a.starts_with(&geminated_b) || geminated_b.starts_with(a))
}

/// Remove every small tsu from the reading. Filtering ALL っ (not just a
/// final one) is intentional: on readings place っ only in gemination
/// position, so any っ-bearing reading joins the family of its
/// dakuten-free base through the prefix checks below.
fn strip_gemination(reading: &str) -> String {
    reading.chars().filter(|&c| c != 'っ').collect()
}

fn split_kept_and_removed(
    readings: &[String],
    mut removed: Vec<usize>,
) -> (Vec<String>, Vec<String>) {
    removed.sort_unstable();
    let mut kept = Vec::new();
    let mut removed_texts = Vec::new();
    for (index, reading) in readings.iter().enumerate() {
        if removed.binary_search(&index).is_ok() {
            removed_texts.push(reading.clone());
        } else {
            kept.push(reading.clone());
        }
    }
    (kept, removed_texts)
}

fn leading_kanji(text: &str) -> Option<char> {
    match text.chars().next() {
        Some(c) if is_kanji(c) => Some(c),
        _ => None,
    }
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let (root_a, root_b) = (find_root(parent, a), find_root(parent, b));
    if root_a != root_b {
        parent[root_b] = root_a;
    }
}

fn find_root(parent: &[usize], mut node: usize) -> usize {
    while parent[node] != node {
        node = parent[node];
    }
    node
}

fn root_groups(parent: &[usize]) -> Vec<Vec<usize>> {
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for node in 0..parent.len() {
        groups
            .entry(find_root(parent, node))
            .or_default()
            .push(node);
    }
    groups.into_values().collect()
}

fn log_report(total_kanji: usize, details: &[CollapseDetail]) {
    let on_removed: usize = details.iter().map(|detail| detail.on_removed.len()).sum();
    let kun_removed: usize = details.iter().map(|detail| detail.kun_removed.len()).sum();
    tracing::info!("Kanji processed: {}", total_kanji);
    tracing::info!("Kanji affected: {}", details.len());
    tracing::info!("On readings removed: {}", on_removed);
    tracing::info!("Kun readings removed: {}", kun_removed);
    for detail in details {
        tracing::info!(
            "  {} — removed on: {:?}, removed kun: {:?}",
            detail.kanji,
            detail.on_removed,
            detail.kun_removed
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture corpus: 見-words for the kun triple, 十/世 words for the on
    /// groups, 間-words for the go-on/kan-on doublet, 日 words for the
    /// rendaku split.
    const CORPUS: &str = "\
見る|みる|0:み
見える|みえる|0:み
見せる|みせる|0:み
一つ|ひとつ|0:ひと
一に|いつに|0:いつ
日|ひ|0:ひ
日々|ひび|0:ひ;1:び
十分|じゅうぶん|0:じゅう
十五|じゅうご|0:じゅう
十回|じゅっかい|0:じゅっ
世話|せわ|0:せ
世間|せけん|0:せ
世紀|せいき|0:せい
間|ま|0:ま
時間|じかん|0:じかん
人間|にんげん|1:げん
";

    fn corpus() -> KanjiCorpusIndex {
        let dictionary = FuriganaDictionary::from_text(CORPUS).unwrap();
        KanjiCorpusIndex::build(&dictionary)
    }

    fn list(readings: &[&str]) -> Vec<String> {
        readings.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn kun_readings_sharing_unambiguous_body_collapse_to_popular_word_reading() {
        // Arrange: all three 見 readings carry the body み; 見る is a
        // popular word, so it represents the group.
        let index = corpus();
        let kun = list(&["みる", "みえる", "みせる"]);
        let popular = list(&["見る"]);

        // Act
        let (kept, removed) = collapse_kun_readings("見", &kun, &index, &popular);

        // Assert
        assert_eq!(kept, vec!["みる"]);
        assert_eq!(removed, vec!["みえる", "みせる"]);
    }

    #[test]
    fn kun_group_without_popular_match_keeps_shortest_form() {
        // Arrange
        let index = corpus();
        let kun = list(&["みる", "みえる", "みせる"]);

        // Act
        let (kept, removed) = collapse_kun_readings("見", &kun, &index, &[]);

        // Assert
        assert_eq!(kept, vec!["みる"]);
        assert_eq!(removed, vec!["みえる", "みせる"]);
    }

    #[test]
    fn kun_reading_with_corpus_absent_form_is_untouched() {
        // Arrange: みやける has no corpus word, so it stays ungrouped.
        let index = corpus();
        let kun = list(&["みる", "みえる", "みやける"]);

        // Act
        let (kept, removed) = collapse_kun_readings("見", &kun, &index, &[]);

        // Assert
        assert_eq!(kept, vec!["みる", "みやける"]);
        assert_eq!(removed, vec!["みえる"]);
    }

    #[test]
    fn rendaku_kun_readings_without_shared_body_stay_all() {
        // Arrange: ひ reads 日 in the corpus directly, び only appears
        // inside 日々 (full reading ひび ≠ び), か has no word at all.
        let index = corpus();
        let kun = list(&["ひ", "び", "か"]);

        // Act
        let (kept, removed) = collapse_kun_readings("日", &kun, &index, &[]);

        // Assert
        assert_eq!(kept, vec!["ひ", "び", "か"]);
        assert!(removed.is_empty());
    }

    #[test]
    fn on_prefix_variants_collapse_to_most_frequent_surface() {
        // Arrange: 世 reads せ in two corpus words (世話, 世間) and せい in
        // one (世紀) — the prefix pair keeps the more frequent surface.
        let index = corpus();
        let on = list(&["セイ", "セ"]);

        // Act
        let (kept, removed) = collapse_on_readings("世", &on, &index);

        // Assert
        assert_eq!(kept, vec!["セ"]);
        assert_eq!(removed, vec!["セイ"]);
    }

    #[test]
    fn on_gemination_variants_collapse_to_base_reading() {
        // Arrange: じゅう is demonstrated by 十分/十五, じゅっ by 十回,
        // じっ by nothing — all three are one gemination family.
        let index = corpus();
        let on = list(&["ジュウ", "ジッ", "ジュッ"]);

        // Act
        let (kept, removed) = collapse_on_readings("十", &on, &index);

        // Assert
        assert_eq!(kept, vec!["ジュウ"]);
        assert_eq!(removed, vec!["ジッ", "ジュッ"]);
    }

    #[test]
    fn on_go_on_kan_on_doublet_is_untouched() {
        // Arrange: 間 reads かん (時間) and げん (人間) — the pair differs
        // in the first mora, neither grouping rule may fire.
        let index = corpus();
        let on = list(&["カン", "ケン"]);

        // Act
        let (kept, removed) = collapse_on_readings("間", &on, &index);

        // Assert
        assert_eq!(kept, vec!["カン", "ケン"]);
        assert!(removed.is_empty());
    }

    #[test]
    fn single_reading_kanji_is_untouched() {
        // Arrange
        let index = corpus();
        let on = list(&["イン"]);
        let kun = list(&["のむ"]);

        // Act
        let (on_kept, on_removed) = collapse_on_readings("飲", &on, &index);
        let (kun_kept, kun_removed) = collapse_kun_readings("飲", &kun, &index, &[]);

        // Assert
        assert_eq!(on_kept, vec!["イン"]);
        assert_eq!(kun_kept, vec!["のむ"]);
        assert!(on_removed.is_empty() && kun_removed.is_empty());
    }

    // --- Integration tests: the file boundary of run_collapse_kanji_readings.

    const FURIGANA: &str = "\
見る|みる|0:み
見える|みえる|0:み
見せる|みせる|0:み
間|ま|0:ま
時間|じかん|0:じかん
人間|にんげん|1:げん
";

    /// Two kanji: 見 collapses (body み three times), 間 stays (go-on/kan-on
    /// doublet). 見 carries stale frequency keys for the removed forms and
    /// a description field that must survive the rewrite.
    fn fixture_dictionary() -> String {
        r#"{
  "kanji": [
    {
      "kanji": "見",
      "on_readings": ["ケン"],
      "kun_readings": ["みる", "みえる", "みせる"],
      "description_en": ["see"],
      "popular_words": ["見る"],
      "reading_frequencies": {"ケン": 800, "みる": 1011, "みえる": 1012, "みせる": 300}
    },
    {
      "kanji": "間",
      "on_readings": ["カン", "ケン"],
      "kun_readings": ["あいだ", "ま"]
    }
  ]
}"#
        .to_string()
    }

    fn temp_input(contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("origa-collapse-test-{}", ulid::Ulid::new()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("kanji.json");
        fs::write(&path, contents).unwrap();
        path
    }

    fn temp_corpus() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("origa-collapse-test-{}", ulid::Ulid::new()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("JmdictFurigana.txt");
        fs::write(&path, FURIGANA).unwrap();
        path
    }

    fn entry(contents: &str, kanji: &str) -> serde_json::Value {
        let value: serde_json::Value = serde_json::from_str(contents).unwrap();
        value["kanji"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["kanji"] == kanji)
            .unwrap()
            .clone()
    }

    #[test]
    fn apply_rewrites_file_with_backup_and_stale_frequencies_removed() {
        // Arrange
        let original = fixture_dictionary();
        let input = temp_input(&original);
        let furigana = temp_corpus();

        // Act
        run_collapse_kanji_readings(input.clone(), furigana, true).unwrap();

        // Assert: the collapsed lists landed in the file.
        let rewritten = fs::read_to_string(&input).unwrap();
        let mi = entry(&rewritten, "見");
        assert_eq!(mi["kun_readings"], serde_json::json!(["みる"]));
        assert_eq!(mi["on_readings"], serde_json::json!(["ケン"]));
        assert_eq!(
            mi["reading_frequencies"],
            serde_json::json!({"ケン": 800, "みる": 1011})
        );
        assert_eq!(mi["description_en"], serde_json::json!(["see"]));
        // The go-on/kan-on doublet stayed untouched.
        let ma = entry(&rewritten, "間");
        assert_eq!(ma["on_readings"], serde_json::json!(["カン", "ケン"]));
        // The backup carries the pre-collapse bytes.
        let backup = fs::read_to_string(input.with_extension("json.bak")).unwrap();
        assert_eq!(backup, original);
    }

    #[test]
    fn repeated_apply_is_byte_identical() {
        // Arrange
        let input = temp_input(&fixture_dictionary());
        let furigana = temp_corpus();

        // Act
        run_collapse_kanji_readings(input.clone(), furigana.clone(), true).unwrap();
        let first = fs::read_to_string(&input).unwrap();
        run_collapse_kanji_readings(input.clone(), furigana, true).unwrap();
        let second = fs::read_to_string(&input).unwrap();

        // Assert
        assert_eq!(first, second);
    }

    #[test]
    fn dry_run_writes_nothing() {
        // Arrange
        let original = fixture_dictionary();
        let input = temp_input(&original);
        let furigana = temp_corpus();

        // Act
        run_collapse_kanji_readings(input.clone(), furigana, false).unwrap();

        // Assert
        assert_eq!(fs::read_to_string(&input).unwrap(), original);
        assert!(!input.with_extension("json.bak").exists());
    }

    #[test]
    fn missing_input_fails_loudly() {
        // Arrange
        let input = std::env::temp_dir().join(format!(
            "origa-collapse-test-{}-absent.json",
            ulid::Ulid::new()
        ));
        let furigana = temp_corpus();

        // Act
        let result = run_collapse_kanji_readings(input, furigana, false);

        // Assert
        assert!(result.is_err());
    }
}
