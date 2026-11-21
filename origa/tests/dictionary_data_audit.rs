//! Regression guard for the dictionary data layer: kana tokens whose Sudachi
//! lemma carries an ateji kanji orthography must resolve to the functional
//! word translation, not to the kanji-glyph gloss.
//!
//! Repro (11.11 owner report): the さて quiz card showed the kanji 扠 with the
//! СОЮЗ category and the verb gloss «вставлять / втыкать» — the dictionary
//! entry for the ateji key 扠 was generated from the glyph, not from the
//! lemma. Vocabulary cards build on `orthographic_base_form`, so the entry
//! under the ateji key is the only lookup path.
//!
//! Corpus-dependent: every test skips with a `[skip]` marker when the local
//! CDN corpus is absent (mirrors well_known_sets_audit / corpus_audit
//! policy). Run explicitly with:
//!   `cargo test -p origa --test dictionary_data_audit`

use origa::dictionary::vocabulary::get_translations;
use origa::domain::{NativeLanguage, PartOfSpeech, tokenize_text};

#[path = "translation_smoke/bootstrap.rs"]
mod bootstrap;

use bootstrap::{cdn_path, ensure_all_dictionaries};

fn corpus_available() -> bool {
    if ensure_all_dictionaries() {
        return true;
    }
    eprintln!(
        "[skip] dictionary_data_audit: local corpus not found at {}",
        cdn_path(&["dictionary"]).display()
    );
    false
}

/// さて must tokenize as a conjunction whose base form is the ateji key 扠 —
/// the only key the vocabulary card looks up.
#[test]
fn sate_token_resolves_to_ateji_key_with_conjunction_pos() {
    // Arrange
    if !corpus_available() {
        return;
    }

    // Act
    let tokens = tokenize_text("さて").expect("tokenization must succeed");

    // Assert
    let token = tokens.first().expect("at least one token for さて");
    assert_eq!(token.orthographic_surface_form(), "さて");
    assert_eq!(token.orthographic_base_form(), "扠");
    assert_eq!(*token.part_of_speech(), PartOfSpeech::Conjunction);
}

/// The dictionary entry for the ateji key 扠 must carry the conjunction gloss
/// (owner pin: «итак / ну / что ж»), never the kanji-glyph verb gloss.
#[test]
fn ateji_key_sate_has_conjunction_gloss_not_kanji_verb_gloss() {
    // Arrange
    if !corpus_available() {
        return;
    }
    let lang = NativeLanguage::Russian;

    // Act
    let translations = get_translations("扠", &lang).expect("扠 entry must exist");

    // Assert
    assert_eq!(
        translations.first().map(String::as_str),
        Some("итак"),
        "conjunction gloss must lead the entry: {translations:?}"
    );
    for banned in ["втыкать", "вставлять"] {
        assert!(
            translations.iter().all(|t| !t.contains(banned)),
            "kanji-glyph verb gloss must not leak into the entry: {translations:?}"
        );
    }
}

/// 詰まり is polysemous: conjunction つまり and the noun «затор» family share
/// the key, and card answers render the whole list. The conjunction gloss
/// must lead the list; the noun glosses must survive the prepend.
#[test]
fn tumari_key_leads_with_conjunction_gloss_and_keeps_noun_glosses() {
    // Arrange
    if !corpus_available() {
        return;
    }
    let lang = NativeLanguage::Russian;

    // Act
    let translations = get_translations("詰まり", &lang).expect("詰まり entry must exist");

    // Assert
    let first = translations.first().expect("non-empty translation list");
    assert!(
        first.contains("то есть"),
        "conjunction gloss must lead the list: {first}"
    );
    let joined = translations.join(" / ");
    assert!(
        joined.contains("затор") || joined.contains("тупик") || joined.contains("застой"),
        "noun glosses must survive the conjunction prepend: {translations:?}"
    );
}

/// Discovery probe (manual, `--ignored --nocapture`): POS of tokens whose
/// base form lands on the suspect keys 詰る / 故. Verb/noun lemmas keep their
/// glosses; conjunction/adverb lemmas would need a data fix.
#[test]
#[ignore]
fn pos_probe_for_suspect_ateji_keys() {
    if !corpus_available() {
        return;
    }
    for text in ["詰る", "つまる", "詰まり", "つまり", "ゆえ", "故に", "故"] {
        let tokens = tokenize_text(text).expect("tokenization must succeed");
        for token in tokens {
            println!(
                "{text:>8} → surface {} / base {} / POS {:?}",
                token.orthographic_surface_form(),
                token.orthographic_base_form(),
                token.part_of_speech()
            );
        }
    }
}
