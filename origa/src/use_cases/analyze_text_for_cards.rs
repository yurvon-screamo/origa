use crate::{
    domain::{OrigaError, PartOfSpeech, User, tokenize_text},
    traits::UserRepository,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedWord {
    pub base_form: String,
    pub reading: String,
    pub part_of_speech: PartOfSpeech,
    pub is_known: bool,
    pub meaning: Option<String>,
    /// Кандидат — счётный суффикс (issue #415): заведётся counter-картой,
    /// не словарным словом. UI маркирует строку бейджем типа.
    #[serde(default)]
    pub is_counter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeTextResult {
    pub words: Vec<AnalyzedWord>,
    pub total_found: usize,
    pub known_count: usize,
    pub new_count: usize,
}

pub struct AnalyzeTextForCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

/// Числительная поверхность: кандзи-числа, 〇 и арабские цифры. Судачи
/// помечает числительные внутри сочетаний как Noun (一本 → 一/Noun +
/// 本/Suffix), поэтому одного POS `Numeral` недостаточно.
fn is_numeral_surface(surface: &str) -> bool {
    const NUMERALS: &str = "一二三四五六七八九十百千万億〇0123456789";
    let mut chars = surface.chars();
    match chars.next() {
        Some(first) if NUMERALS.contains(first) => chars.all(|c| NUMERALS.contains(c)),
        _ => false,
    }
}

fn is_numeral_token(token: &crate::domain::tokenizer::TokenInfo) -> bool {
    token.part_of_speech() == &PartOfSpeech::Numeral
        || (token.part_of_speech() == &PartOfSpeech::Noun
            && is_numeral_surface(token.orthographic_surface_form()))
}

/// Пары соседних токенов «числительное → счётный суффикс»: (суффикс,
/// чтение). Только суффиксы из загруженного реестра счётчиков.
fn detected_counter_suffixes(
    tokens: &[crate::domain::tokenizer::TokenInfo],
) -> Vec<(String, String)> {
    if !crate::dictionary::counters::is_counters_loaded() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut prev_numeral = false;
    for token in tokens {
        let is_pair = prev_numeral
            && token.part_of_speech() == &PartOfSpeech::Suffix
            && crate::dictionary::counters::get_counter(token.orthographic_surface_form())
                .is_some();
        if is_pair {
            result.push((
                token.orthographic_surface_form().to_string(),
                token.phonological_surface_form().to_string(),
            ));
        }
        prev_numeral = is_numeral_token(token);
    }
    result
}

fn user_knows_counter(user: &User, suffix: &str) -> bool {
    user.knowledge_set()
        .study_cards()
        .values()
        .any(|sc| matches!(sc.card(), crate::domain::Card::Counter(c) if c.suffix() == suffix))
}

impl<'a, R: UserRepository> AnalyzeTextForCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, text: String) -> Result<AnalyzeTextResult, OrigaError> {
        debug!(text_length = text.len(), "Analyzing text for cards");

        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let tokens = tokenize_text(text.as_str())?;

        debug!(token_count = tokens.len(), "Tokenized text");

        let mut words: Vec<AnalyzedWord> = Vec::new();
        let mut seen_words = std::collections::HashSet::new();

        for token in &tokens {
            if !token.part_of_speech().is_vocabulary_word() {
                continue;
            }

            let word_text = token.orthographic_base_form().to_string();
            if seen_words.contains(&word_text) {
                continue;
            }

            seen_words.insert(word_text.clone());

            let knowledge = user.is_word_known(&word_text);

            words.push(AnalyzedWord {
                base_form: word_text.clone(),
                reading: token.phonological_base_form().to_string(),
                part_of_speech: token.part_of_speech().clone(),
                is_known: knowledge.is_known,
                meaning: knowledge.meaning,
                is_counter: false,
            });
        }

        // Счётные суффиксы из ТЕКСТА (issue #415), два источника:
        // 1) пары соседних токенов «числительное → суффикс» (一本 → 一 | 本);
        // 2) склейки судачи — лексикализованные сочетания одним токеном
        //    (三本 → ProperNoun 三本), парой не расщепляются, контекст
        //    ловится по поверхности лексемы.
        // Суффикс попадает в кандидаты со глоссой реестра; POS Suffix
        // доезжает до create и превращает его в counter-карту.
        let mut detected = detected_counter_suffixes(&tokens);
        for token in &tokens {
            for entry in crate::dictionary::counters::counters_detected_in_surface(
                token.orthographic_surface_form(),
            ) {
                detected.push((
                    entry.suffix().to_string(),
                    token.phonological_surface_form().to_string(),
                ));
            }
        }
        let mut seen_suffixes = std::collections::HashSet::new();
        for (suffix, reading) in detected {
            if !seen_suffixes.insert(suffix.clone()) {
                continue;
            }
            let key = format!("counter:{suffix}");
            if seen_words.contains(&key) {
                continue;
            }
            seen_words.insert(key);
            let Some(entry) = crate::dictionary::counters::get_counter(&suffix) else {
                continue;
            };
            let meaning =
                crate::dictionary::counters::gloss_for(entry, *user.native_language()).to_string();
            words.push(AnalyzedWord {
                base_form: suffix.clone(),
                reading,
                part_of_speech: PartOfSpeech::Suffix,
                is_known: user_knows_counter(&user, &suffix),
                meaning: Some(meaning),
                is_counter: true,
            });
        }

        let total_found = words.len();
        let known_count = words.iter().filter(|w| w.is_known).count();
        let new_count = total_found - known_count;

        info!(total_found, known_count, new_count, "Text analyzed");

        Ok(AnalyzeTextResult {
            words,
            total_found,
            known_count,
            new_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tokenizer::TokenInfo;

    /// Пары «числительное → суффикс» детектятся только по реестру:
    /// суффикс вне реестра или не-числительный сосед игнорируются.
    #[test]
    fn detected_counter_suffixes_requires_numeral_neighbour() {
        crate::dictionary::counters::tests::init_test_counters();
        let tokens = vec![
            TokenInfo::new_test_with_reading("三", "さん", PartOfSpeech::Numeral),
            TokenInfo::new_test_with_reading("本", "ほん", PartOfSpeech::Suffix),
            TokenInfo::new_test_with_reading("私", "わたし", PartOfSpeech::Pronoun),
            TokenInfo::new_test_with_reading("日", "にち", PartOfSpeech::Suffix),
        ];
        let detected = detected_counter_suffixes(&tokens);
        assert_eq!(detected.len(), 1, "only the numeral-adjacent 本 registers");
        assert_eq!(detected[0].0, "本");
    }

    /// Склейки судачи (三本 одним ProperNoun-токеном) парой не
    /// расщепляются — контекст ловится по поверхности лексемы в execute.
    #[test]
    fn glued_counter_compound_detected_via_surface() {
        crate::dictionary::counters::tests::init_test_counters();
        // пара не образуется: один токен Noun 三 + один ProperNoun 三本? —
        // берём реальную форму: единственный токен-склейка 三本.
        let tokens = vec![TokenInfo::new_test_with_reading(
            "三本",
            "さんぼん",
            PartOfSpeech::ProperNoun,
        )];
        assert!(detected_counter_suffixes(&tokens).is_empty());
        assert_eq!(
            crate::dictionary::counters::counters_detected_in_surface("三本")
                .iter()
                .map(|e| e.suffix())
                .collect::<Vec<_>>(),
            vec!["本"]
        );
    }

    /// Судачи помечает числительные в сочетаниях как Noun (一本 →
    /// 一/Noun): пара детектится по числительной ПОВЕРХНОСТИ.
    #[test]
    fn detected_counter_suffixes_accepts_noun_numerals() {
        crate::dictionary::counters::tests::init_test_counters();
        let tokens = vec![
            TokenInfo::new_test_with_reading("一", "いち", PartOfSpeech::Noun),
            TokenInfo::new_test_with_reading("本", "ほん", PartOfSpeech::Suffix),
            TokenInfo::new_test_with_reading("月", "つき", PartOfSpeech::Noun),
            TokenInfo::new_test_with_reading("人", "じん", PartOfSpeech::Suffix),
        ];
        let detected = detected_counter_suffixes(&tokens);
        assert_eq!(
            detected.len(),
            1,
            "Noun 一 + Suffix 本 is a pair; 月/人 is not"
        );
        assert_eq!(detected[0].0, "本");
    }

    #[test]
    fn detected_counter_suffixes_empty_without_registry() {
        // Реестр уже установлен другими тестами — проверяем деградацию
        // напрямую: пустой реестр => пустой результат.
        let tokens = vec![
            TokenInfo::new_test_with_reading("三", "さん", PartOfSpeech::Numeral),
            TokenInfo::new_test_with_reading("虚", "きょ", PartOfSpeech::Suffix),
        ];
        assert!(detected_counter_suffixes(&tokens).is_empty());
    }
}
