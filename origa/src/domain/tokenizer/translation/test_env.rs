//! Shared dictionary fixture for translation integration tests.
//!
//! Loads the three dictionaries the translation pipeline reads (tokenizer
//! SudachiDict, vocabulary chunks, grammar rules) from the local corpus
//! (see [`crate::corpus`]). Deliberately lighter than the crate-wide test
//! fixture: it does not load kanji/radicals, which the translation pipeline
//! never reads — the skip guard mirrors the fixture, not the whole family.
//!
//! Corpus absence is a skip, not a panic: tests enter through the
//! `translation_dictionaries_or_skip!` macro ([`crate::corpus`]).

use crate::corpus::{cdn_dir, corpus_absent};
use crate::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use crate::dictionary::vocabulary::{VocabularyChunkData, init_vocabulary, is_vocabulary_loaded};
use crate::domain::{DictionaryData, init_dictionary, is_dictionary_loaded};

pub(super) fn ensure_dictionaries() {
    ensure_tokenizer_dictionary();
    ensure_vocabulary_dictionary();
    ensure_grammar_dictionary();
}

/// True when the three dictionaries of [`ensure_dictionaries`] are loaded.
pub(super) fn translation_dictionaries_ready() -> bool {
    is_dictionary_loaded() && is_vocabulary_loaded() && is_grammar_loaded()
}

fn ensure_tokenizer_dictionary() {
    if is_dictionary_loaded() {
        return;
    }

    use flate2::read::DeflateDecoder;
    use std::io::Read;

    let dict_dir = cdn_dir()
        .join("dictionaries")
        .join(crate::domain::SUDACHIDICT_DIR);
    if corpus_absent(&dict_dir, "SudachiDict") {
        return;
    }

    let decompress = |data: Vec<u8>| -> Vec<u8> {
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    };

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
}

fn ensure_vocabulary_dictionary() {
    if is_vocabulary_loaded() {
        return;
    }

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

    // Tolerate the OnceLock init race: a concurrent test thread may have
    // initialized the vocabulary first with identical data. Only a genuinely
    // unset dictionary indicates a real failure; preserve the error detail
    // so data regressions stay debuggable.
    if let Err(e) = init_vocabulary(vocab_data) {
        if !is_vocabulary_loaded() {
            panic!("vocabulary dictionary failed to load: {:?}", e);
        }
    }
}

fn ensure_grammar_dictionary() {
    if is_grammar_loaded() {
        return;
    }

    let grammar_path = cdn_dir().join("grammar").join("grammar_v3.json");
    if corpus_absent(&grammar_path, "grammar rules") {
        return;
    }

    let grammar_json =
        std::fs::read_to_string(grammar_path).expect("grammar_v3.json exists (probed above)");
    // Same OnceLock race tolerance as ensure_vocabulary_dictionary.
    if let Err(e) = init_grammar(GrammarData { grammar_json }) {
        if !is_grammar_loaded() {
            panic!("grammar dictionary failed to load: {:?}", e);
        }
    }
}
