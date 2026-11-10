//! Crate-level test fixtures shared by domain and use_cases tests.
//!
//! Lives outside `use_cases` so that domain tests can initialize the
//! process-wide dictionaries without reaching into the layer above them
//! (the domain → use_cases inversion flagged in issue #796). The fixture
//! depends only on `crate::dictionary::*`, `crate::domain::*` and std.
//!
//! Corpus absence is a skip, not a panic: every loader probes its corpus
//! entry first and bails out via [`crate::corpus`] when unseeded — calling
//! tests then skip through the `*_or_skip!` macros. Setup helpers that
//! cannot early-return re-check the `is_*_loaded` predicates instead.

use std::io::Read;
use std::sync::Once;

use flate2::read::DeflateDecoder;

use crate::corpus::{cdn_dir, corpus_absent};
use crate::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use crate::dictionary::kanji::{KanjiData, init_kanji, is_kanji_loaded};
use crate::dictionary::phrase::{init_phrase_index, is_phrases_loaded};
use crate::dictionary::radical::{RadicalData, init_radicals, is_radicals_loaded};
use crate::dictionary::vocabulary::{VocabularyChunkData, init_vocabulary, is_vocabulary_loaded};
use crate::domain::{DictionaryData, init_dictionary, is_dictionary_loaded};

static TOKENIZER_INIT: Once = Once::new();
static VOCABULARY_INIT: Once = Once::new();
static KANJI_INIT: Once = Once::new();
static RADICALS_INIT: Once = Once::new();
static GRAMMAR_INIT: Once = Once::new();
static PHRASE_INDEX_INIT: Once = Once::new();

fn decompress(data: Vec<u8>) -> Vec<u8> {
    let mut decoder = DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

/// True when all five dictionaries of [`init_real_dictionaries`] are loaded.
pub(crate) fn real_dictionaries_ready() -> bool {
    is_dictionary_loaded()
        && is_vocabulary_loaded()
        && is_kanji_loaded()
        && is_radicals_loaded()
        && is_grammar_loaded()
}

pub(crate) fn init_real_dictionaries() {
    init_tokenizer_dictionary();
    init_vocabulary_dictionary();
    init_kanji_dictionary();
    init_radicals_dictionary();
    init_grammar_rules();
}

/// Loads the corpus phrase index into the process-wide `PHRASE_INDEX`.
/// Required by lesson-builder tests because `init_real_dictionaries` intentionally
/// does not pull the phrase index (production loads it lazily from the CDN).
pub(crate) fn init_phrase_index_from_cdn() {
    PHRASE_INDEX_INIT.call_once(|| {
        if is_phrases_loaded() {
            return;
        }

        let index_path = cdn_dir().join("phrases").join("phrase_index.json");
        if corpus_absent(&index_path, "phrase index") {
            return;
        }

        let index_json =
            std::fs::read_to_string(&index_path).expect("phrase index exists (probed above)");
        if let Err(e) = init_phrase_index(&index_json) {
            tracing::warn!("Failed to init phrase index: {:?}", e);
        }
    });
}

fn init_tokenizer_dictionary() {
    TOKENIZER_INIT.call_once(|| {
        if is_dictionary_loaded() {
            return;
        }

        let dict_dir = cdn_dir()
            .join("dictionaries")
            .join(crate::domain::SUDACHIDICT_DIR);
        if corpus_absent(&dict_dir, "SudachiDict") {
            return;
        }

        let read_file = |name: &str| std::fs::read(dict_dir.join(name)).unwrap();

        let data = DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_trie: decompress(read_file("dict.trie")),
            dict_vals_idx: decompress(read_file("dict.valsidx")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        };

        init_dictionary(data).unwrap();
    });
}

fn init_vocabulary_dictionary() {
    VOCABULARY_INIT.call_once(|| {
        let vocab_dir = cdn_dir().join("dictionary");
        if corpus_absent(&vocab_dir.join("chunk_01.json"), "vocabulary chunks") {
            return;
        }

        let read_chunk = |name: &str| {
            std::fs::read_to_string(vocab_dir.join(name))
                .unwrap_or_else(|e| panic!("failed to read vocabulary chunk {name}: {e}"))
        };
        let vocab_data = VocabularyChunkData {
            chunk_01: read_chunk("chunk_01.json"),
            chunk_02: read_chunk("chunk_02.json"),
            chunk_03: read_chunk("chunk_03.json"),
            chunk_04: read_chunk("chunk_04.json"),
            chunk_05: read_chunk("chunk_05.json"),
            chunk_06: read_chunk("chunk_06.json"),
            chunk_07: read_chunk("chunk_07.json"),
            chunk_08: read_chunk("chunk_08.json"),
            chunk_09: read_chunk("chunk_09.json"),
            chunk_10: read_chunk("chunk_10.json"),
            chunk_11: read_chunk("chunk_11.json"),
        };
        // Tolerate the OnceLock race: a concurrent test thread may have
        // installed the dictionary first (identical real chunks). A panic
        // here would poison the Once for every other caller — only a
        // genuinely unset dictionary indicates a real failure (same
        // contract as domain::tokenizer::translation::test_env).
        if let Err(e) = init_vocabulary(vocab_data) {
            if !is_vocabulary_loaded() {
                panic!("Failed to init vocabulary dictionary: {e:?}");
            }
        }
    });
}

fn init_kanji_dictionary() {
    KANJI_INIT.call_once(|| {
        if is_kanji_loaded() {
            return;
        }

        let kanji_path = cdn_dir().join("dictionary").join("kanji.json");
        if corpus_absent(&kanji_path, "kanji dictionary") {
            return;
        }

        let kanji_json =
            std::fs::read_to_string(&kanji_path).expect("kanji.json exists (probed above)");
        init_kanji(KanjiData { kanji_json }).expect("Failed to init kanji dictionary");
    });
}

fn init_radicals_dictionary() {
    RADICALS_INIT.call_once(|| {
        if is_radicals_loaded() {
            return;
        }

        let radicals_path = cdn_dir().join("dictionary").join("radicals.json");
        if corpus_absent(&radicals_path, "radicals dictionary") {
            return;
        }

        let radicals_json =
            std::fs::read_to_string(&radicals_path).expect("radicals.json exists (probed above)");
        init_radicals(RadicalData { radicals_json }).expect("Failed to init radicals dictionary");
    });
}

fn init_grammar_rules() {
    GRAMMAR_INIT.call_once(|| {
        if is_grammar_loaded() {
            return;
        }

        let grammar_path = cdn_dir().join("grammar").join("grammar_v3.json");
        if corpus_absent(&grammar_path, "grammar rules") {
            return;
        }

        let grammar_json =
            std::fs::read_to_string(&grammar_path).expect("grammar_v3.json exists (probed above)");
        if let Err(e) = init_grammar(GrammarData { grammar_json }) {
            tracing::warn!("Failed to init grammar rules: {:?}", e);
        }
    });
}
