use crate::i18n::{I18nContext, Locale};
use origa::domain::PartOfSpeech;

pub fn part_of_speech_label(pos: PartOfSpeech, i18n: &I18nContext<Locale>) -> String {
    let keys = i18n.get_keys_untracked().lesson();
    match pos {
        PartOfSpeech::Verb => keys.pos_verb().inner().to_string(),
        PartOfSpeech::Noun => keys.pos_noun().inner().to_string(),
        PartOfSpeech::IAdjective => keys.pos_i_adjective().inner().to_string(),
        PartOfSpeech::NaAdjective => keys.pos_na_adjective().inner().to_string(),
        PartOfSpeech::Adverb => keys.pos_adverb().inner().to_string(),
        PartOfSpeech::PreNounAdjectival => keys.pos_pre_noun_adjectival().inner().to_string(),
        PartOfSpeech::Conjunction => keys.pos_conjunction().inner().to_string(),
        PartOfSpeech::Interjection => keys.pos_interjection().inner().to_string(),
        PartOfSpeech::Prefix => keys.pos_prefix().inner().to_string(),
        PartOfSpeech::Suffix => keys.pos_suffix().inner().to_string(),
        PartOfSpeech::Particle => keys.pos_particle().inner().to_string(),
        PartOfSpeech::AuxiliaryVerb => keys.pos_auxiliary_verb().inner().to_string(),
        PartOfSpeech::Pronoun => keys.pos_pronoun().inner().to_string(),
        PartOfSpeech::ProperNoun => keys.pos_proper_noun().inner().to_string(),
        PartOfSpeech::Numeral => keys.pos_numeral().inner().to_string(),
        PartOfSpeech::Determiner => keys.pos_determiner().inner().to_string(),
        PartOfSpeech::Unspecified => keys.pos_unspecified().inner().to_string(),
        PartOfSpeech::Other => keys.pos_other().inner().to_string(),
        PartOfSpeech::Symbol => keys.pos_symbol().inner().to_string(),
        PartOfSpeech::Whitespace => keys.pos_whitespace().inner().to_string(),
        PartOfSpeech::AuxiliarySymbol => keys.pos_auxiliary_symbol().inner().to_string(),
    }
}
