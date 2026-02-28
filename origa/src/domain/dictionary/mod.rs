pub mod kanji;
pub mod radical;
pub mod vocabulary;

pub use kanji::{
    KANJI_DICTIONARY, KanjiData, KanjiInfo, PopularWord, get_kanji_info, get_kanji_list,
    init_kanji_dictionary, is_kanji_loaded,
};
pub use radical::{
    RADICAL_DICTIONARY, RadicalData, RadicalInfo, get_radical_info, init_radical_dictionary,
    is_radical_loaded,
};
pub use vocabulary::{
    VOCABULARY_DICTIONARY, VocabularyChunkData, VocabularyInfo, get_translation,
    init_vocabulary_dictionary, is_vocabulary_loaded,
};
