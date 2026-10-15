//! Панель «Дорогие слова»: доля проблемных слов и топ провалов.

use crate::domain::{Card, CardType, KnowledgeSet, NativeLanguage};

/// Порог «проблемного» слова: два и более провалов за жизнь. Именно
/// накопительный `lapses`, а не `consecutive_again`: последний сбрасывается
/// при спавне добивания и стрик длиннее одного не видит.
const TROUBLED_LAPSES: u32 = 2;
/// Размер топа дорогих слов.
pub const HARDEST_TOP_SIZE: usize = 3;

/// Одно слово из топа дорогих.
#[derive(Debug, Clone, PartialEq)]
pub struct HardWord {
    pub japanese: String,
    pub meaning: String,
    pub lapses: u32,
}

/// Дорогие слова: доля проблемных слов и топ самых провальных.
#[derive(Debug, Clone, PartialEq)]
pub struct HardestWordsInsight {
    /// Знаменатель «1 из N слов».
    pub one_in: u32,
    pub troubled_count: usize,
    pub studied_total: usize,
    pub top: Vec<HardWord>,
}

struct WordTrouble {
    lapses: u32,
    difficulty: f64,
}

fn word_trouble(study_card: &crate::domain::StudyCard) -> Option<WordTrouble> {
    let memory = study_card.memory();
    if memory.is_new() {
        return None;
    }
    Some(WordTrouble {
        lapses: memory.lapses(),
        difficulty: memory.difficulty().map(|d| d.value()).unwrap_or(0.0),
    })
}

/// #749 панель «Дорогие слова»: доля проблемных слов (два и более провалов
/// за жизнь среди изученных слов) и топ самых провальных.
pub fn compute_hardest_words(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> Option<HardestWordsInsight> {
    let mut studied_total = 0usize;
    let mut troubled: Vec<(&crate::domain::StudyCard, WordTrouble)> = Vec::new();

    for study_card in knowledge_set.study_cards().values() {
        if matches!(
            CardType::from(study_card.card()),
            CardType::Phrase | CardType::Example
        ) {
            continue;
        }
        let Some(trouble) = word_trouble(study_card) else {
            continue;
        };
        studied_total += 1;
        if trouble.lapses >= TROUBLED_LAPSES {
            troubled.push((study_card, trouble));
        }
    }

    if studied_total == 0 || troubled.is_empty() {
        return None;
    }

    troubled.sort_by(|a, b| {
        b.1.lapses
            .cmp(&a.1.lapses)
            .then(b.1.difficulty.total_cmp(&a.1.difficulty))
    });

    // Карта без перевода пропускается в топе (панель рендерит сколько есть);
    // в подсчёте troubled/total она участвует всегда.
    let top: Vec<HardWord> = troubled
        .iter()
        .take(HARDEST_TOP_SIZE)
        .filter_map(|(study_card, trouble)| hard_word_of(study_card, lang, trouble))
        .collect();

    let troubled_count = troubled.len();
    let one_in = (studied_total as f64 / troubled_count as f64).ceil() as u32;

    Some(HardestWordsInsight {
        one_in,
        troubled_count,
        studied_total,
        top,
    })
}

/// Тексты карты; карта без перевода даёт None (не попадает в топ).
fn hard_word_of(
    study_card: &crate::domain::StudyCard,
    lang: &NativeLanguage,
    trouble: &WordTrouble,
) -> Option<HardWord> {
    let card = study_card.card();
    let japanese = match card {
        Card::Vocabulary(v) => v.word().text().to_string(),
        Card::Kanji(k) => k.kanji().text().to_string(),
        Card::Grammar(g) => g.rule_id().to_string(),
        Card::Counter(c) => c.suffix().to_string(),
        Card::Phrase(_) | Card::Example(_) => return None,
    };
    let meaning = card.answer(lang).ok()?.text_projection();

    Some(HardWord {
        japanese,
        meaning,
        lapses: trouble.lapses,
    })
}
