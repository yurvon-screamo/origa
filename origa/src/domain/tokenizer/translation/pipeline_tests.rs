//! Pipeline-level integration tests: translation fallbacks, counter
//! labels, compound splits and tokenizer characterization phrases that
//! exercise `lookup_tokens_translations` end to end.

use super::test_env::ensure_dictionaries;
use super::*;
use crate::domain::tokenizer::tokenize_text;

#[test]
fn should_translate_bakari() {
    ensure_dictionaries();
    let text = "ばかり";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
    let bakari = results.iter().find(|t| t.surface_form.contains("ばかり"));
    assert!(bakari.is_some(), "「ばかり」token should exist");
    assert!(
        bakari.unwrap().translation.is_some(),
        "「ばかり」should have a translation"
    );
}
#[test]
fn should_translate_uwa_interjection() {
    ensure_dictionaries();
    let text = "うわー";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
    let uwa = results.iter().find(|t| t.surface_form.contains("うわ"));
    assert!(uwa.is_some(), "「うわー」token should exist");
    assert!(
        uwa.unwrap().translation.is_some(),
        "「うわー」should have a translation"
    );
}
#[test]
fn should_translate_souiu_compound() {
    ensure_dictionaries();
    let text = "そういう";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
    let souiu = results.iter().find(|t| t.surface_form.contains("そういう"));
    assert!(souiu.is_some(), "「そういう」token should exist");
    assert!(
        souiu.unwrap().translation.is_some(),
        "「そういう」should have a translation"
    );
}
#[test]
fn should_translate_hodo() {
    ensure_dictionaries();
    let text = "ほど";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
    let hodo = results.iter().find(|t| t.surface_form.contains("ほど"));
    assert!(hodo.is_some(), "「ほど」token should exist");
    assert!(
        hodo.unwrap().translation.is_some(),
        "「ほど」should have a translation"
    );
}
#[test]
fn suffix_counter_token_gets_localized_label_and_gloss() {
    crate::dictionary::counters::tests::init_test_counters();
    let tokens = vec![TokenInfo::new_test_with_reading(
        "本",
        "ほん",
        PartOfSpeech::Suffix,
    )];
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, "三本");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].grammar_label.as_deref(), Some("Счётный суффикс"));
    assert_eq!(
        results[0].grammar_description.as_deref(),
        Some("длинные предметы")
    );
}
#[test]
fn standalone_noun_counter_surface_gets_no_counter_label() {
    // POS-гейт: 本 как существительное — словарный путь без метки.
    crate::dictionary::counters::tests::init_test_counters();
    let tokens = vec![TokenInfo::new_test_with_reading(
        "本",
        "ほん",
        PartOfSpeech::Noun,
    )];
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, "本");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].grammar_label.is_none(),
        "Noun must not carry the counter label"
    );
}
#[test]
fn counter_label_falls_back_to_english_gloss_for_partial_locales() {
    crate::dictionary::counters::tests::init_test_counters();
    let tokens = vec![TokenInfo::new_test_with_reading(
        "本",
        "ほん",
        PartOfSpeech::Suffix,
    )];
    // KO глосс в фикстуре есть — но цепочка обязана работать и на
    // частичном реестре: проверяем через English-локаль напрямую.
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, "三本");
    assert_eq!(results[0].grammar_label.as_deref(), Some("Counter"));
    assert_eq!(
        results[0].grammar_description.as_deref(),
        Some("long objects")
    );
}
// Verification for issue #178 P-8: characterization of how Lindera tokenizes
// "さあついたよ、ここだ". Result is correct as-is (interjection + verb past +
// particle + punct + pronoun + copula) — wontfix with this note.
#[test]
fn tokenize_saa_tsuita_yo_kokoda_returns_expected_tokens() {
    ensure_dictionaries();
    let text = "さあついたよ、ここだ";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

    let surfaces: Vec<&str> = results.iter().map(|t| t.surface_form.as_str()).collect();
    assert!(
        surfaces.contains(&"さあ"),
        "「さあ」interjection should be tokenized separately, got: {:?}",
        surfaces
    );
    assert!(
        surfaces
            .iter()
            .any(|s| s.contains("ついた") || s.contains("つい")),
        "「ついた」verb past should be tokenized, got: {:?}",
        surfaces
    );
    assert!(
        surfaces.contains(&"よ"),
        "「よ」particle should be tokenized, got: {:?}",
        surfaces
    );
    assert!(
        surfaces.contains(&"ここ"),
        "「ここ」pronoun should be tokenized, got: {:?}",
        surfaces
    );
    assert!(
        surfaces.contains(&"だ"),
        "「だ」copula should be tokenized, got: {:?}",
        surfaces
    );
}
// --- Reproduction / regression tests for translation bugs ---
// These tests were originally written as failing repro-tests to document
// concrete root causes across 10 user-reported phrases. After the 6-slice
// fix they now pass and serve as regression locks. The
// `should_split_mizunashi_compound` test locks the post-processing
// splitter that resolves the 水なし over-merge limitation.

// --- Phrase 1: 人はパンのみで生きるにあらずですわよ ---
// Before the fix, Lindera split the classical negative あらず into あら
// (Verb, base 有る) + ず (AuxiliaryVerb), and the ず token surfaced base_form
// 'ぬ' (Lindera's lemma), which was absent from the vocabulary dictionary —
// so translation was None and no grammar rule labeled the ず classical-negative
// auxiliary. This test locks in the vocabulary entry for ぬ that resolves it.
#[test]
fn should_label_or_translate_classical_negative_zu_auxiliary() {
    ensure_dictionaries();
    let text = "人はパンのみで生きるにあらずですわよ";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let zu = results
        .iter()
        .find(|t| t.surface_form == "ず")
        .expect("「ず」auxiliary token should exist");
    assert!(
        zu.translation.is_some() || zu.grammar_label.is_some(),
        "classical negative 「ず」should carry a translation or grammar_label, got: {:?}",
        zu
    );
}
// --- Phrase 1: わ (sentence-final particle, feminine/emphatic) ---
// Before the fix, the standalone 「わ」 particle had no vocabulary entry, so
// translation was None. This test locks in the added translation.
#[test]
fn should_translate_wa_sentence_final_particle() {
    ensure_dictionaries();
    let text = "生きるにあらずですわよ";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let wa = results
        .iter()
        .find(|t| t.surface_form == "わ" && t.pos == PartOfSpeech::Particle)
        .expect("「わ」particle token should exist");
    assert!(
        wa.translation.is_some(),
        "sentence-final 「わ」particle should have a translation, got: {:?}",
        wa
    );
}
// --- Phrase 2: 水なしでは生きていけない ---
// Regression test for the post-processing splitter. Lindera over-merges
// 水なし ("without water") into a single ProperNoun token (reading ミズナシ)
// with no vocabulary translation. The splitter in `tokenize_text` detects
// ProperNoun tokens ending in a known productive suffix (currently なし)
// and splits them into separate Noun tokens so each gets translated.
// Previous approach (user_dictionary.csv + なし) broke E2E via furigana
// rendering; this code-only fix avoids touching the build binary.
#[test]
fn should_split_mizunashi_compound() {
    ensure_dictionaries();
    let text = "水なしでは生きていけない";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let merged = results.iter().find(|t| t.surface_form == "水なし");
    assert!(
        merged.is_none(),
        "「水なし」should be split into separate tokens, got merged token: {:?}",
        merged
    );

    let mizu = results
        .iter()
        .find(|t| t.surface_form == "水" && t.pos == PartOfSpeech::Noun)
        .expect("「水」noun should exist after split");
    // Split must expose 水's dictionary entry — the whole point of the fix.
    assert!(
        mizu.translation.is_some(),
        "「水」should carry a translation after split, got: {:?}",
        mizu
    );
    // Reading is derived from the merged token's phonological form
    // (ミズナシ → strip suffix-length chars → ミズ), not the surface kanji.
    assert_ne!(
        mizu.reading, "水",
        "「水」reading must be katakana ミズ, not surface kanji"
    );

    // なし has no standalone dictionary entry in the current vocabulary —
    // the split still produces the token (needed for furigana/glossary
    // alignment), but translation absence is a dictionary-data concern,
    // not a splitter concern.
    let nashi = results
        .iter()
        .find(|t| t.surface_form == "なし")
        .expect("「なし」token should exist after split");
    assert_ne!(
        nashi.reading, "なし",
        "「なし」reading must be katakana ナシ derived from merged token"
    );
}
// --- 努力くらいはしてみるよ ---
// Lindera over-merges 努力くらい into a single ProperNoun (ドリョクライ),
// leaving くらい with no token at all. The splitter must split it into
// 努力 (Noun) + くらい (Particle). The くらい grammar rule has AND-semantics
// keyword groups ([["くらい"],["ぐらい"]]) that require BOTH forms in the
// text, so くらい does NOT receive a grammar_label here — the split itself
// is the fix; the grammar_label data gap is a separate cdn/ concern.
#[test]
fn should_split_doryokukurai_compound() {
    ensure_dictionaries();
    let text = "努力くらいはしてみるよ";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let merged = results.iter().find(|t| t.surface_form == "努力くらい");
    assert!(
        merged.is_none(),
        "「努力くらい」should be split into separate tokens, got merged token: {:?}",
        merged
    );

    let _doryoku = results
        .iter()
        .find(|t| t.surface_form == "努力" && t.pos == PartOfSpeech::Noun)
        .expect("「努力」noun should exist after split");

    let kurai = results
        .iter()
        .find(|t| t.surface_form == "くらい" && t.pos == PartOfSpeech::Particle)
        .expect("「くらい」particle token should exist after split");
    assert_ne!(
        kurai.reading, "くらい",
        "「くらい」reading must be a katakana reading derived from the merged token, not the hiragana surface"
    );
}
// --- Phrase 6: カッコイイの！ ---
// Before the fix, the katakana adjective カッコイイ ("cool") was split into
// カッコ (格好) + イイ (良い), so the single-word meaning "cool/stylish" was
// lost. This test locks in the user-dictionary entry that keeps it one token.
#[test]
fn should_tokenize_kakkoii_katakana_as_single_token() {
    ensure_dictionaries();
    let text = "カッコイイの！";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let kakkoii = results.iter().find(|t| t.surface_form == "カッコイイ");
    assert!(
        kakkoii.is_some(),
        "「カッコイイ」should be a single token, got surfaces: {:?}",
        results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
    );
}
// --- Phrase 7: かっこいいですね ---
// Before the fix, the hiragana adjective かっこいい WAS present in the
// vocabulary dictionary (chunk_11), but Lindera split it into かっこ (base 格好)
// + いい (base 良い). Translation lookup used base_form, so the existing
// かっこいい entry was never consulted and the "cool/stylish" meaning was lost.
// This test locks in the user-dictionary entry that keeps it one token.
#[test]
fn should_use_kakkoii_dictionary_entry_for_hiragana_compound() {
    ensure_dictionaries();
    let text = "かっこいいですね";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let uses_entry = results
        .iter()
        .any(|t| t.surface_form == "かっこいい" || t.base_form == "かっこいい");
    assert!(
        uses_entry,
        "「かっこいい」should resolve to its dictionary entry, got (surface, base): {:?}",
        results
            .iter()
            .map(|t| (&t.surface_form, &t.base_form))
            .collect::<Vec<_>>()
    );
}
// --- Phrase 8: 心をひとつに ---
// Before the fix, ひとつ ("one") had a vocabulary entry but was split into
// ひと (base 一) + つ (suffix), so the single-word entry was never used.
// This test locks in the user-dictionary entry that keeps it one token.
#[test]
fn should_tokenize_hitotsu_as_single_token() {
    ensure_dictionaries();
    let text = "心をひとつに";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let hitotsu = results
        .iter()
        .any(|t| t.surface_form == "ひとつ" || t.surface_form == "一つ");
    assert!(
        hitotsu,
        "「ひとつ」should be a single token, got surfaces: {:?}",
        results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
    );
}
// --- Phrase 9: やれやれ、無駄なことだよ ---
// Before the fix, やれやれ ("good grief") had a vocabulary entry but was split
// into two やれ tokens, so the interjection entry was never used and each half
// was translated with the imperative meaning instead. This test locks in the
// user-dictionary entry that keeps it one token.
#[test]
fn should_tokenize_yareyare_as_single_token() {
    ensure_dictionaries();
    let text = "やれやれ、無駄なことだよ。";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let yareyare = results.iter().any(|t| t.surface_form == "やれやれ");
    assert!(
        yareyare,
        "「やれやれ」should be a single token, got surfaces: {:?}",
        results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
    );
}
// --- Phrase 9: 起きるものか ---
// Before the fix, 〜ものか ("no way / absolutely not", rhetorical exclamation)
// was present in the vocabulary dictionary (ru: "ни в коем случае!"), but
// Lindera split ものか into もの (base 物) + か, so the rhetorical-meaning entry
// was never consulted and もの resolved to the unrelated "thing" translation
// instead. This test locks in the user-dictionary entry that keeps it one token.
#[test]
fn should_tokenize_monoka_as_single_token() {
    ensure_dictionaries();
    let text = "起きるものか";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let monoka = results.iter().any(|t| t.surface_form == "ものか");
    assert!(
        monoka,
        "「ものか」should be a single token so its dictionary entry is used, got surfaces: {:?}",
        results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
    );
}
// --- Phrase 10: ふふ、ストーブついてますし ---
// Before the fix, the interjection ふふ ("hehe") had no vocabulary entry, so
// translation was None. This test locks in the added translation.
#[test]
fn should_translate_fufu_interjection() {
    ensure_dictionaries();
    let text = "ふふ、ストーブついてますし";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let fufu = results
        .iter()
        .find(|t| t.surface_form == "ふふ")
        .expect("「ふふ」interjection token should exist");
    assert!(
        fufu.translation.is_some(),
        "「ふふ」interjection should have a translation, got: {:?}",
        fufu
    );
}
// Characterization test: the input `言っイッてみただけ` contains an OCR/furigana
// artifact (the katakana イッ inside a hiragana verb form). Lindera cannot
// reconstruct the canonical te-form 言って here, but it must still recognize
// the verb stem 言っ and surface its dictionary form 言う so the translation
// pipeline can work. The katakana イッ being read as the numeral 一 is a known
// tokenizer/dictionary limitation and is out of scope for translation.rs.
#[test]
fn should_still_extract_iu_base_form_from_mixed_kana_kanji_input() {
    ensure_dictionaries();
    let text = "言っイッてみただけ";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

    let iu = results.iter().find(|t| t.base_form == "言う");
    assert!(
        iu.is_some(),
        "Should find 言う base_form in '{}', tokens: {:?}",
        text,
        results
            .iter()
            .map(|t| (&t.surface_form, &t.base_form))
            .collect::<Vec<_>>()
    );
}
// Particle 「し」 lists reason particle. Its base_form resolves to a homonym
// (為る "to do" / する-stem), so without the homonym blacklist the particle
// would be translated as the verb instead of being left untranslated.
#[test]
fn should_suppress_homonym_translation_for_particle_shi() {
    ensure_dictionaries();
    let text = "ストーブついてますし、大丈夫ですよ";
    let tokens = tokenize_text(text).unwrap();
    let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

    let shi = results
        .iter()
        .find(|t| t.surface_form == "し" && t.pos == PartOfSpeech::Particle)
        .expect("Particle 「し」 token should exist (tokenization confirmed by repro test)");
    assert!(
        shi.translation.is_none(),
        "Particle 「し」must not be translated via homonym (為る=делать), got: {:?}",
        shi.translation
    );
}
