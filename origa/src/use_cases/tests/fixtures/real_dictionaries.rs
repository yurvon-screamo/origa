use std::io::Read;
use std::sync::Once;

use flate2::read::DeflateDecoder;

use super::get_public_dir;
use crate::dictionary::grammar::{init_grammar, is_grammar_loaded, GrammarData};
use crate::dictionary::kanji::{init_kanji, is_kanji_loaded, KanjiData};
use crate::dictionary::radical::{init_radicals, is_radicals_loaded, RadicalData};
use crate::dictionary::vocabulary::{init_vocabulary, VocabularyChunkData};
use crate::domain::{init_dictionary, is_dictionary_loaded, DictionaryData};

static INIT: Once = Once::new();

fn decompress(data: Vec<u8>) -> Vec<u8> {
    let mut decoder = DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

pub fn init_real_dictionaries() {
    INIT.call_once(|| {
        init_tokenizer_dictionary();
        init_vocabulary_dictionary();
        init_kanji_dictionary();
        init_radicals_dictionary();
        init_grammar_rules();
    });
}

fn init_tokenizer_dictionary() {
    if is_dictionary_loaded() {
        return;
    }

    let public_dir = get_public_dir();
    let dict_dir = public_dir.join("dictionaries").join("unidic");

    let read_file = |name: &str| std::fs::read(dict_dir.join(name)).unwrap();

    let data = DictionaryData {
        char_def: decompress(read_file("char_def.bin")),
        matrix: decompress(read_file("matrix.mtx")),
        dict_da: decompress(read_file("dict.da")),
        dict_vals: decompress(read_file("dict.vals")),
        unk: decompress(read_file("unk.bin")),
        words_idx: decompress(read_file("dict.wordsidx")),
        words: decompress(read_file("dict.words")),
        metadata: read_file("metadata.json"),
    };

    init_dictionary(data).unwrap();
}

fn init_vocabulary_dictionary() {
    let public_dir = get_public_dir();

    let vocab_dir = public_dir.join("dictionary").join("vocabulary");
    let vocab_data = VocabularyChunkData {
        chunk_01: std::fs::read_to_string(vocab_dir.join("chunk_01.json"))
            .expect("Failed to read vocabulary chunk_01.json"),
        chunk_02: std::fs::read_to_string(vocab_dir.join("chunk_02.json"))
            .expect("Failed to read vocabulary chunk_02.json"),
        chunk_03: std::fs::read_to_string(vocab_dir.join("chunk_03.json"))
            .expect("Failed to read vocabulary chunk_03.json"),
        chunk_04: std::fs::read_to_string(vocab_dir.join("chunk_04.json"))
            .expect("Failed to read vocabulary chunk_04.json"),
        chunk_05: std::fs::read_to_string(vocab_dir.join("chunk_05.json"))
            .expect("Failed to read vocabulary chunk_05.json"),
        chunk_06: std::fs::read_to_string(vocab_dir.join("chunk_06.json"))
            .expect("Failed to read vocabulary chunk_06.json"),
        chunk_07: std::fs::read_to_string(vocab_dir.join("chunk_07.json"))
            .expect("Failed to read vocabulary chunk_07.json"),
        chunk_08: std::fs::read_to_string(vocab_dir.join("chunk_08.json"))
            .expect("Failed to read vocabulary chunk_08.json"),
        chunk_09: std::fs::read_to_string(vocab_dir.join("chunk_09.json"))
            .expect("Failed to read vocabulary chunk_09.json"),
        chunk_10: std::fs::read_to_string(vocab_dir.join("chunk_10.json"))
            .expect("Failed to read vocabulary chunk_10.json"),
        chunk_11: std::fs::read_to_string(vocab_dir.join("chunk_11.json"))
            .expect("Failed to read vocabulary chunk_11.json"),
    };
    init_vocabulary(vocab_data).expect("Failed to init vocabulary dictionary");
}

fn init_kanji_dictionary() {
    if is_kanji_loaded() {
        return;
    }

    let public_dir = get_public_dir();

    let kanji_path = public_dir.join("dictionary").join("kanji.json");
    let kanji_data = KanjiData {
        kanji_json: std::fs::read_to_string(&kanji_path).expect("Failed to read kanji.json"),
    };
    init_kanji(kanji_data).expect("Failed to init kanji dictionary");
}

fn init_radicals_dictionary() {
    if is_radicals_loaded() {
        return;
    }

    let public_dir = get_public_dir();
    let radicals_path = public_dir.join("dictionary").join("radicals.json");
    let radicals_data = RadicalData {
        radicals_json: std::fs::read_to_string(&radicals_path)
            .expect("Failed to read radicals.json"),
    };
    init_radicals(radicals_data).expect("Failed to init radicals dictionary");
}

fn init_grammar_rules() {
    if is_grammar_loaded() {
        return;
    }

    let public_dir = get_public_dir();
    let grammar_path = public_dir.join("grammar").join("grammar.json");

    let grammar_json = match std::fs::read_to_string(&grammar_path) {
        Ok(content) => content,
        Err(e) => {
            tracing::warn!(
                "Grammar file not found, skipping grammar initialization: {}",
                e
            );
            return;
        },
    };

    let grammar_data = GrammarData { grammar_json };
    if let Err(e) = init_grammar(grammar_data) {
        tracing::warn!("Failed to init grammar rules: {:?}", e);
    }
}
