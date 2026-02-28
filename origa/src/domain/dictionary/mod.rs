pub mod kanji;
pub mod radical;
pub mod vocabulary;

pub use kanji::{
    get_kanji_info, get_kanji_list, init_kanji_dictionary, is_kanji_loaded, KanjiData, KanjiInfo,
    PopularWord, KANJI_DICTIONARY,
};
pub use radical::{
    get_radical_info, init_radical_dictionary, is_radical_loaded, RadicalData, RadicalInfo,
    RADICAL_DICTIONARY,
};
pub use vocabulary::{
    get_translation, init_vocabulary_dictionary, is_vocabulary_loaded, VocabularyChunkData,
    VocabularyInfo, VOCABULARY_DICTIONARY,
};
