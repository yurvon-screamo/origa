use std::sync::Once;

use super::get_public_dir;
use crate::domain::{
    GrammarData, KanjiData, VocabularyChunkData, init_grammar, init_kanji, init_vocabulary,
};

static INIT: Once = Once::new();

pub fn init_real_dictionaries() {
    INIT.call_once(|| {
        let public_dir = get_public_dir();

        let vocab_dir = public_dir
            .join("dictionary")
            .join("vocabulary");
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

        let kanji_path = public_dir
            .join("dictionary")
            .join("kanji.json");
        let kanji_data = KanjiData {
            kanji_json: std::fs::read_to_string(&kanji_path).expect("Failed to read kanji.json"),
        };
        init_kanji(kanji_data).expect("Failed to init kanji dictionary");

        let grammar_path = public_dir
            .join("grammar")
            .join("grammar.json");
        let grammar_data = GrammarData {
            grammar_json: std::fs::read_to_string(&grammar_path)
                .expect("Failed to read grammar.json"),
        };
        init_grammar(grammar_data).expect("Failed to init grammar rules");
    });
}
