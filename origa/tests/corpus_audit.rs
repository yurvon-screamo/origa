//! Extended corpus audit over a 10k random sample drawn from ALL
//! `cdn/phrases/data/pNNNN.json` chunks (~156k phrases total).
//!
//! **Role.** This is a manual discovery instrument, NOT a CI regression guard.
//! All entry points are `#[ignore]` because they depend on the gitignored
//! `cdn/` dataset (156k phrases + dictionaries) that is not present in CI.
//! Run explicitly with:
//!   `cargo test -p origa --test corpus_audit -- --nocapture --ignored`
//!
//! The regression lock for the one code bug it surfaced (T1, kanji leaking
//! into phonological reading for Unknown-word kanji tokens) lives in the
//! in-crate unit test `should_not_use_kanji_surface_as_phonological_reading_for_unknown_kanji`
//! in `domain/tokenizer/mod.rs`. This file is retained so the audit can be
//! re-run after dictionary/grammar data changes to surface new tokenizer-level
//! anomalies that never reach `TokenTranslation`.
//!
//! Unlike `translation_smoke` (single chunk, translation-level only), this
//! audit ALSO inspects the tokenizer-level `TokenInfo` output of
//! `tokenize_text` for bugs that never reach `TokenTranslation`:
//!
//! - **T1** `ReadingContainsKanji` — a phonological_*_form carries kanji.
//!   Readings MUST be kana; kanji there means the reading derivation broke.
//! - **T2** `StarBaseOrSurface` — base/surface is literally "*": Lindera's
//!   empty marker leaked through `token_to_token_info`.
//! - **T3** `LexemeHyphenInBase` — base carries a "-english" suffix:
//!   `lexeme.split_once('-')` failed for a non-ASCII hyphen variant.
//! - **T4** `ReadingHasAscii` — phonological form contains ASCII letters for
//!   a vocabulary word (corrupted reading).
//! - **T5** `EmptySurface` — empty orthographic_surface_form.
//! - **T6** `UnspecifiedPos` — `PartOfSpeech::Unspecified`: the POS string
//!   from Lindera did not round-trip through `FromStr`.
//! - **T7** `TokenizeError` — `tokenize_text` returned `Err`.

use std::collections::HashMap;

use origa::domain::{JapaneseChar, PartOfSpeech, TokenInfo, tokenize_text};

#[path = "translation_smoke/bootstrap.rs"]
mod bootstrap;

use bootstrap::{cdn_path, ensure_all_dictionaries};

// ---------- corpus sampling ----------

#[derive(serde::Deserialize)]
struct PhraseRaw {
    x: String,
}

/// Deterministic splitmix64 — keeps the 10k sample reproducible across runs so
/// any regression introduced after a fix re-surfaces at the same phrase index.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn load_all_phrases() -> Vec<String> {
    let dir = cdn_path(&["phrases", "data"]);
    let mut entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read phrases/data: {e}"))
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let p = e.path();
            let name = p.file_name()?.to_string_lossy().to_string();
            if name.starts_with('p') && name.ends_with(".json") {
                Some(p)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    entries.sort();

    let mut all = Vec::new();
    for path in &entries {
        let body = match std::fs::read_to_string(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let parsed: Vec<PhraseRaw> = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };
        all.extend(parsed.into_iter().map(|p| p.x));
    }
    all
}

fn sample_phrases(phrases: &[String], n: usize) -> Vec<String> {
    if phrases.len() <= n {
        return phrases.to_vec();
    }
    // Fisher–Yates with splitmix64 — deterministic.
    let mut idx: Vec<usize> = (0..phrases.len()).collect();
    let mut state: u64 = 0xC0FF_0042_DEAD_BEEF;
    for i in (1..idx.len()).rev() {
        let j = (splitmix64(&mut state) as usize) % (i + 1);
        idx.swap(i, j);
    }
    idx.into_iter()
        .take(n)
        .map(|i| phrases[i].clone())
        .collect()
}

// ---------- audit ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AuditKind {
    TokenizeError,
    ReadingContainsKanji,
    StarBaseOrSurface,
    LexemeHyphenInBase,
    ReadingHasAscii,
    EmptySurface,
    UnspecifiedPos,
}

impl AuditKind {
    fn code(self) -> &'static str {
        match self {
            AuditKind::TokenizeError => "T7",
            AuditKind::ReadingContainsKanji => "T1",
            AuditKind::StarBaseOrSurface => "T2",
            AuditKind::LexemeHyphenInBase => "T3",
            AuditKind::ReadingHasAscii => "T4",
            AuditKind::EmptySurface => "T5",
            AuditKind::UnspecifiedPos => "T6",
        }
    }
}

#[derive(Debug, Clone)]
struct AuditReport {
    kind: AuditKind,
    phrase_idx: usize,
    phrase_text: String,
    token_idx: usize,
    surface: String,
    base: String,
    reading: String,
    pos: PartOfSpeech,
    detail: String,
}

fn contains_kanji(s: &str) -> bool {
    s.chars().any(|c| c.is_kanji())
}

fn contains_ascii_letter(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_alphabetic())
}

fn contains_hyphen_suffix(s: &str) -> bool {
    // The lexeme-split in token_to_token_info splits on ASCII '-'. Catch bases
    // that still carry a "-word" tail: either a non-ASCII dash (– —) or a
    // suffix beginning with another separator.
    s.contains('-') || s.contains('–') || s.contains('—')
}

fn audit_token(
    token: &TokenInfo,
    phrase_idx: usize,
    phrase_text: &str,
    token_idx: usize,
    out: &mut Vec<AuditReport>,
) {
    let surface = token.orthographic_surface_form();
    let base = token.orthographic_base_form();
    let phon_base = token.phonological_base_form();
    let phon_surface = token.phonological_surface_form();
    let pos = token.part_of_speech().clone();

    let push = |kind: AuditKind, detail: String, field_val: &str, out: &mut Vec<AuditReport>| {
        out.push(AuditReport {
            kind,
            phrase_idx,
            phrase_text: phrase_text.to_string(),
            token_idx,
            surface: surface.to_string(),
            base: base.to_string(),
            reading: field_val.to_string(),
            pos: pos.clone(),
            detail,
        });
    };

    if surface.is_empty() {
        push(AuditKind::EmptySurface, "empty surface".into(), "", out);
    }
    if surface == "*" || base == "*" {
        push(
            AuditKind::StarBaseOrSurface,
            format!("star leak: surface='{surface}' base='{base}'"),
            "",
            out,
        );
    }
    if contains_hyphen_suffix(base) {
        push(
            AuditKind::LexemeHyphenInBase,
            format!("base carries hyphen/lexeme tail: '{base}'"),
            base,
            out,
        );
    }
    if pos == PartOfSpeech::Unspecified {
        push(
            AuditKind::UnspecifiedPos,
            "POS parsed as Unspecified".into(),
            "",
            out,
        );
    }

    let is_vocab = pos.is_vocabulary_word();
    if is_vocab {
        for (label, field) in [("phon_base", phon_base), ("phon_surface", phon_surface)] {
            if contains_kanji(field) {
                push(
                    AuditKind::ReadingContainsKanji,
                    format!("{label}='{field}' contains kanji"),
                    field,
                    out,
                );
            }
        }
        // ASCII in a reading is only legitimate for western loanword surface
        // (rare); flag it so a human can confirm.
        if contains_ascii_letter(phon_base) || contains_ascii_letter(phon_surface) {
            push(
                AuditKind::ReadingHasAscii,
                format!("ascii in reading: phon_base='{phon_base}' phon_surface='{phon_surface}'"),
                phon_base,
                out,
            );
        }
    }
}

fn run_audit(sample: &[String]) -> Vec<AuditReport> {
    let mut reports = Vec::new();
    for (idx, text) in sample.iter().enumerate() {
        match tokenize_text(text) {
            Ok(tokens) => {
                for (token_idx, token) in tokens.iter().enumerate() {
                    audit_token(token, idx, text, token_idx, &mut reports);
                }
            },
            Err(err) => reports.push(AuditReport {
                kind: AuditKind::TokenizeError,
                phrase_idx: idx,
                phrase_text: text.clone(),
                token_idx: 0,
                surface: String::new(),
                base: String::new(),
                reading: String::new(),
                pos: PartOfSpeech::Unspecified,
                detail: format!("tokenize_text failed: {err:?}"),
            }),
        }
    }
    reports
}

fn print_audit(reports: &[AuditReport], sample_size: usize) {
    eprintln!();
    eprintln!("=== CORPUS AUDIT (sample={sample_size}) ===");
    eprintln!("total problems: {}", reports.len());

    let mut counts: HashMap<AuditKind, usize> = HashMap::new();
    for r in reports {
        *counts.entry(r.kind).or_default() += 1;
    }
    let mut kinds: Vec<AuditKind> = counts.keys().copied().collect();
    kinds.sort_by_key(|k| k.code());

    for kind in kinds {
        let n = counts[&kind];
        eprintln!("{}\t{}", kind.code(), n);
        for r in reports.iter().filter(|r| r.kind == kind).take(8) {
            eprintln!(
                "    [phrase#{} tok#{}] 「{}」(base 「{}」 reading 「{}」 pos {:?}) — {} | phrase: {}",
                r.phrase_idx,
                r.token_idx,
                r.surface,
                r.base,
                r.reading,
                r.pos,
                r.detail,
                r.phrase_text.chars().take(60).collect::<String>()
            );
        }
        if kind == AuditKind::ReadingContainsKanji {
            let mut uniq: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
            for r in reports.iter().filter(|r| r.kind == kind) {
                uniq.insert(r.base.as_str());
            }
            eprintln!("    unique base forms with kanji reading: {:?}", uniq);
        }
    }
}

#[test]
#[ignore = "corpus audit: scans ALL phrases for kanji-in-reading, dump unique forms"]
fn audit_full_corpus_kanji_reading() {
    if !ensure_all_dictionaries() {
        eprintln!(
            "skip: cdn artifacts absent \
             (cdn/ is gitignored; restore via scripts/deploy_cdn.py)"
        );
        return;
    }
    let all = load_all_phrases();
    eprintln!("loaded {} total phrases", all.len());
    let reports = run_audit(&all);
    print_audit(&reports, all.len());
}

#[test]
#[ignore = "corpus audit: samples 10k phrases, dump anomaly summary, run explicitly"]
fn audit_ten_k_random_phrases() {
    if !ensure_all_dictionaries() {
        eprintln!(
            "skip: cdn artifacts absent \
             (cdn/ is gitignored; restore via scripts/deploy_cdn.py)"
        );
        return;
    }
    let all = load_all_phrases();
    eprintln!("loaded {} total phrases", all.len());
    let sample = sample_phrases(&all, 10_000);
    let reports = run_audit(&sample);
    print_audit(&reports, sample.len());
}

// ---------- translation-level audit ----------

use origa::domain::{NativeLanguage, TokenTranslation, lookup_tokens_translations};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TransKind {
    TokenizeError,
    EmptyBase,
    EmptyReadingForKanjiSurface,
    ReadingContainsKanji,
    VocabNoTranslation,
    ConjugatedNoGrammarLabel,
    ParticleWithContentTranslation,
    GrammarLabelOnUnconjugatedNoun,
}

impl TransKind {
    fn code(self) -> &'static str {
        match self {
            TransKind::TokenizeError => "TF1",
            TransKind::EmptyBase => "TF4",
            TransKind::EmptyReadingForKanjiSurface => "TR0",
            TransKind::ReadingContainsKanji => "TR1",
            TransKind::VocabNoTranslation => "TW1",
            TransKind::ConjugatedNoGrammarLabel => "TW2",
            TransKind::ParticleWithContentTranslation => "TR2",
            TransKind::GrammarLabelOnUnconjugatedNoun => "TR3",
        }
    }
}

#[derive(Debug, Clone)]
struct TransReport {
    kind: TransKind,
    phrase_idx: usize,
    token_idx: usize,
    surface: String,
    base: String,
    reading: String,
    pos: PartOfSpeech,
    text: String,
    translation: Option<String>,
}

fn is_primary_vocab(pos: &PartOfSpeech) -> bool {
    matches!(
        pos,
        PartOfSpeech::Noun | PartOfSpeech::Verb | PartOfSpeech::IAdjective
    )
}

fn classify_translation(
    t: &TokenTranslation,
    phrase_idx: usize,
    token_idx: usize,
    text: &str,
    out: &mut Vec<TransReport>,
) {
    let push = |kind: TransKind, out: &mut Vec<TransReport>| {
        out.push(TransReport {
            kind,
            phrase_idx,
            token_idx,
            surface: t.surface_form.clone(),
            base: t.base_form.clone(),
            reading: t.reading.clone(),
            pos: t.pos.clone(),
            text: text.to_string(),
            translation: t.translation.clone(),
        });
    };

    if t.base_form.is_empty() {
        push(TransKind::EmptyBase, out);
    }
    if t.reading.chars().any(|c| c.is_kanji()) {
        push(TransKind::ReadingContainsKanji, out);
    }
    // After T1 fix, unknown-kanji tokens reach translation with empty reading.
    // Confirm the empty reading is restricted to kanji-bearing surfaces (the
    // only legitimate "no kana reading" case) and never hits kana surfaces.
    if t.reading.is_empty() && t.surface_form.chars().any(|c| c.is_kanji()) {
        push(TransKind::EmptyReadingForKanjiSurface, out);
    }

    // Homonym leak: a particle/auxiliary should never carry a content-word
    // translation unless it is genuinely ambiguous. Particle て/し are
    // blacklisted in translation.rs; this catches any other particle whose
    // base_form resolved to a dictionary content word.
    let is_aux_like = matches!(t.pos, PartOfSpeech::Particle | PartOfSpeech::AuxiliaryVerb);
    if is_aux_like && t.translation.is_some() {
        push(TransKind::ParticleWithContentTranslation, out);
    }

    if is_primary_vocab(&t.pos) {
        if t.translation.is_none() {
            push(TransKind::VocabNoTranslation, out);
        }
        if t.surface_form != t.base_form && t.grammar_label.is_none() {
            push(TransKind::ConjugatedNoGrammarLabel, out);
        }
        // A noun in pure dictionary form (surface == base) carrying a
        // grammar_label is suspicious — common nouns should not pick up
        // grammar labels unless they are grammar nouns (べき, はず, ...).
        if t.pos == PartOfSpeech::Noun && t.surface_form == t.base_form && t.grammar_label.is_some()
        {
            push(TransKind::GrammarLabelOnUnconjugatedNoun, out);
        }
    }
}

fn run_translation_audit(sample: &[String], lang: NativeLanguage) -> Vec<TransReport> {
    let mut reports = Vec::new();
    for (idx, text) in sample.iter().enumerate() {
        match tokenize_text(text) {
            Ok(tokens) => {
                let translations = lookup_tokens_translations(&tokens, &lang, text);
                for (token_idx, t) in translations.iter().enumerate() {
                    classify_translation(t, idx, token_idx, text, &mut reports);
                }
            },
            Err(_) => reports.push(TransReport {
                kind: TransKind::TokenizeError,
                phrase_idx: idx,
                token_idx: 0,
                surface: String::new(),
                base: String::new(),
                reading: String::new(),
                pos: PartOfSpeech::Unspecified,
                text: text.clone(),
                translation: None,
            }),
        }
    }
    reports
}

fn print_translation_audit(reports: &[TransReport], sample_size: usize, lang: &str) {
    eprintln!();
    eprintln!("=== TRANSLATION AUDIT ({lang}, sample={sample_size}) ===");
    eprintln!("total problems: {}", reports.len());

    let mut counts: HashMap<TransKind, usize> = HashMap::new();
    for r in reports {
        *counts.entry(r.kind).or_default() += 1;
    }
    let mut kinds: Vec<TransKind> = counts.keys().copied().collect();
    kinds.sort_by_key(|k| k.code());
    for kind in kinds {
        eprintln!("{}\t{}", kind.code(), counts[&kind]);
        for (shown, r) in reports
            .iter()
            .filter(|r| r.kind == kind)
            .take(6)
            .enumerate()
        {
            eprintln!(
                "    [phrase#{} tok#{}] 「{}」(base 「{}」 reading 「{}」 pos {:?}) tr={:?} gl={:?} | {}",
                r.phrase_idx,
                r.token_idx,
                r.surface,
                r.base,
                r.reading,
                r.pos,
                r.translation,
                "",
                r.text.chars().take(50).collect::<String>()
            );
            let _ = shown;
        }
    }

    // Group ParticleWithContentTranslation by surface to find homonym leaks.
    let tr2 = reports
        .iter()
        .filter(|r| r.kind == TransKind::ParticleWithContentTranslation);
    let mut by_surface: HashMap<String, (usize, Option<String>)> = HashMap::new();
    for r in tr2 {
        let entry = by_surface.entry(r.surface.clone()).or_insert((0, None));
        entry.0 += 1;
        if entry.1.is_none() {
            entry.1 = r.translation.clone();
        }
    }
    let mut ranked: Vec<_> = by_surface.into_iter().collect();
    ranked.sort_by_key(|b| std::cmp::Reverse(b.1.0));
    eprintln!();
    eprintln!("--- TR2 top-20 particle surfaces with content translation ---");
    for (surface, (n, tr)) in ranked.into_iter().take(20) {
        eprintln!("    {n}\t「{surface}」 tr={tr:?}");
    }

    // Group ConjugatedNoGrammarLabel by base_form to surface systematic gaps.
    let w2 = reports
        .iter()
        .filter(|r| r.kind == TransKind::ConjugatedNoGrammarLabel);
    let mut by_base: HashMap<String, usize> = HashMap::new();
    for r in w2 {
        *by_base.entry(r.base.clone()).or_default() += 1;
    }
    let mut ranked: Vec<_> = by_base.into_iter().collect();
    ranked.sort_by_key(|b| std::cmp::Reverse(b.1));
    eprintln!();
    eprintln!("--- TW2 top-20 base forms missing grammar_label ---");
    for (base, n) in ranked.into_iter().take(20) {
        eprintln!("    {n}\t{base}");
    }
}

#[test]
#[ignore = "corpus audit: translation-level over 10k sample, run explicitly"]
fn audit_translation_ten_k_russian() {
    if !ensure_all_dictionaries() {
        eprintln!("skip: cdn artifacts absent");
        return;
    }
    let all = load_all_phrases();
    let sample = sample_phrases(&all, 10_000);
    let reports = run_translation_audit(&sample, NativeLanguage::Russian);
    print_translation_audit(&reports, sample.len(), "ru");
}

// ---------- precise conjugation audit ----------
//
// TW2 above over-reports: it conflates true conjugations (食べる → 食べた, where
// the phonological form changes: タベル → タベタ) with kanji↔kana dictionary-form
// alternation (事 base / こと surface, where phonological_base == phonological_surface
// == コト — same word, just rewritten). The latter is NOT a grammar gap: a
// dictionary-form noun written in kana does not need a grammar_label.
//
// This audit keeps ONLY tokens where the phonological form genuinely changes
// (phonological_base != phonological_surface) — i.e. real conjugations — and
// reports those still missing a grammar_label, grouped by base_form so the
// pattern (which conjugation) is visible.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ConjCategory {
    NounTranslationFix,
    RealContinuativeGap,
    StemBeforeAux,
    HonorificSpecial,
    KuruIrregular,
    Other,
}

impl ConjCategory {
    fn label(self) -> &'static str {
        match self {
            ConjCategory::NounTranslationFix => "noun-translation-fix",
            ConjCategory::RealContinuativeGap => "real-continuative-gap",
            ConjCategory::StemBeforeAux => "stem-before-aux",
            ConjCategory::HonorificSpecial => "honorific-special",
            ConjCategory::KuruIrregular => "kuru-irregular",
            ConjCategory::Other => "other",
        }
    }
}

const HONORIFIC_BASES: &[&str] = &[
    "為さる",
    "なさる",
    "下さる",
    "くださる",
    "御座る",
    "ござる",
    "いらっしゃる",
];

fn classify_conjugation(
    base: &str,
    next_pos: Option<&PartOfSpeech>,
    surface_in_dict: bool,
    base_in_dict: bool,
) -> ConjCategory {
    if base == "来る" {
        return ConjCategory::KuruIrregular;
    }
    if HONORIFIC_BASES.contains(&base) {
        return ConjCategory::HonorificSpecial;
    }
    if matches!(next_pos, Some(PartOfSpeech::AuxiliaryVerb)) {
        return ConjCategory::StemBeforeAux;
    }
    if surface_in_dict {
        return ConjCategory::NounTranslationFix;
    }
    if base_in_dict {
        ConjCategory::RealContinuativeGap
    } else {
        ConjCategory::Other
    }
}

fn run_conjugation_audit(sample: &[String], lang: NativeLanguage) {
    use origa::dictionary::vocabulary::get_translation;

    let mut total_real_conj = 0usize;
    let mut total_no_label = 0usize;
    let mut by_category: HashMap<ConjCategory, usize> = HashMap::new();
    let mut examples_by_category: HashMap<ConjCategory, Vec<String>> = HashMap::new();

    // POS verification gates: collect actual POS for tokens whose POS assumption
    // drives guard logic (て/た/ば) and format_map keys (なさい/ございます/いらっしゃい).
    let mut pos_gate_te_ta_ba: HashMap<&str, HashMap<String, usize>> = HashMap::new();
    let mut pos_gate_honorific: HashMap<&str, HashMap<String, usize>> = HashMap::new();
    const TE_TA_BA_SURFACES: &[&str] = &["て", "た", "ば"];
    const HONORIFIC_SURFACES: &[&str] = &["なさい", "ございます", "いらっしゃい", "ください"];

    for text in sample.iter() {
        let tokens = match tokenize_text(text) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let translations = lookup_tokens_translations(&tokens, &lang, text);
        for (token_idx, token) in tokens.iter().enumerate() {
            let pos = token.part_of_speech();
            // POS verification gates — collect regardless of conjugation status.
            let surface = token.orthographic_surface_form();
            if let Some(key) = TE_TA_BA_SURFACES.iter().find(|s| **s == surface) {
                *pos_gate_te_ta_ba
                    .entry(*key)
                    .or_default()
                    .entry(format!("{pos:?}"))
                    .or_default() += 1;
            }
            if let Some(key) = HONORIFIC_SURFACES.iter().find(|s| **s == surface) {
                *pos_gate_honorific
                    .entry(*key)
                    .or_default()
                    .entry(format!("{pos:?}"))
                    .or_default() += 1;
            }

            if !is_primary_vocab(pos) {
                continue;
            }
            let orth_base = token.orthographic_base_form();
            if orth_base == surface {
                continue;
            }
            let phon_base = token.phonological_base_form();
            let phon_surface = token.phonological_surface_form();
            if phon_base == phon_surface {
                continue;
            }
            total_real_conj += 1;
            let tr = translations.get(token_idx);
            let has_label = tr.is_some_and(|t| t.grammar_label.is_some());
            if has_label {
                continue;
            }
            total_no_label += 1;

            let next_pos = tokens
                .get(token_idx + 1)
                .map(|t| t.part_of_speech().clone());
            let surface_in_dict = get_translation(surface, &lang).is_some();
            let base_in_dict = get_translation(orth_base, &lang).is_some();
            let category =
                classify_conjugation(orth_base, next_pos.as_ref(), surface_in_dict, base_in_dict);
            *by_category.entry(category).or_default() += 1;
            let examples = examples_by_category.entry(category).or_default();
            if examples.len() < 6 {
                examples.push(format!(
                    "{surface}({phon_surface}) <- {orth_base} | next_pos={next_pos:?} | {text}",
                    text = text.chars().take(40).collect::<String>()
                ));
            }
        }
    }

    eprintln!();
    eprintln!(
        "=== CONJUGATION AUDIT (sample={}, lang={:?}) ===",
        sample.len(),
        lang
    );
    eprintln!(
        "real conjugations: {}, without grammar_label: {}",
        total_real_conj, total_no_label
    );

    eprintln!();
    eprintln!("--- category breakdown ---");
    let mut cats: Vec<ConjCategory> = by_category.keys().copied().collect();
    cats.sort_by_key(|c| c.label());
    for cat in cats {
        let n = by_category[&cat];
        let pct = (n as f64 / total_no_label.max(1) as f64) * 100.0;
        eprintln!("    {n:>6} ({pct:5.1}%)  {}", cat.label());
        if let Some(exs) = examples_by_category.get(&cat) {
            for ex in exs.iter().take(4) {
                eprintln!("        {ex}");
            }
        }
    }

    eprintln!();
    eprintln!("--- POS verification gate: て/た/ば ---");
    for s in TE_TA_BA_SURFACES {
        if let Some(pos_counts) = pos_gate_te_ta_ba.get(s) {
            eprintln!("    {s}: {pos_counts:?}");
        }
    }
    eprintln!("--- POS verification gate: honorific surfaces ---");
    for s in HONORIFIC_SURFACES {
        if let Some(pos_counts) = pos_gate_honorific.get(s) {
            eprintln!("    {s}: {pos_counts:?}");
        }
    }
}

#[test]
#[ignore = "corpus audit: precise conjugation gaps over full corpus, run explicitly"]
fn audit_conjugation_full_corpus_russian() {
    if !ensure_all_dictionaries() {
        eprintln!("skip: cdn artifacts absent");
        return;
    }
    let all = load_all_phrases();
    eprintln!("loaded {} total phrases", all.len());
    run_conjugation_audit(&all, NativeLanguage::Russian);
}
