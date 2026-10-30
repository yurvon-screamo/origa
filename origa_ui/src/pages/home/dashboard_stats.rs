use chrono::{Datelike, TimeZone};
use origa::domain::{Card, CardAnswer, CardType, DailyHistoryItem, KnowledgeSet, NativeLanguage};

use crate::pages::shared::format_answer_text;

pub(crate) const RU_MONTHS_SHORT: [&str; 12] = [
    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];
pub(crate) const EN_MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Clone)]
pub struct RecentlyStudiedItem {
    pub card_id: String,
    pub card_type: String,
    pub japanese: String,
    pub meaning: String,
    pub reading: Option<String>,
    pub short_description: Option<String>,
}

/// Один день графика активности (30-дневный MultiLineChart на home).
#[derive(Clone)]
pub struct ActivityDataPoint {
    pub date_label: String,
    pub learned: f64,
    pub in_progress: f64,
    pub new_count: f64,
    pub difficult: f64,
}

/// Активность по последним 30 дням истории: только дни с записями,
/// подписи «14 окт» / «14 Oct» в языке юзера (наработка #528-era).
pub fn compute_30day_chart_data(
    history: &[DailyHistoryItem],
    lang: &NativeLanguage,
) -> Vec<ActivityDataPoint> {
    let mut items: Vec<_> = history.to_vec();
    items.sort_by_key(|a| a.timestamp());

    let start = items.len().saturating_sub(30);
    items
        .into_iter()
        .skip(start)
        .map(|item| {
            let local = chrono::Local.from_utc_datetime(&item.timestamp().naive_utc());
            let day = local.day();
            let month_idx = (local.month0() as usize).min(11);
            let month_str = match lang {
                NativeLanguage::Russian => RU_MONTHS_SHORT[month_idx],
                _ => EN_MONTHS_SHORT[month_idx],
            };
            ActivityDataPoint {
                date_label: format!("{} {}", day, month_str),
                learned: item.known_words() as f64,
                in_progress: item.in_progress_words() as f64,
                new_count: item.new_words() as f64,
                difficult: item.high_difficulty_words() as f64,
            }
        })
        .collect()
}

pub fn compute_studied_today(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> Vec<RecentlyStudiedItem> {
    let today = chrono::Local::now().date_naive();

    let mut cards_today: Vec<_> = knowledge_set
        .study_cards()
        .values()
        .filter(|sc| !matches!(CardType::from(sc.card()), CardType::Phrase))
        .filter(|sc| {
            sc.memory().last_review_date().is_some_and(|last_review| {
                let local_date = chrono::Local
                    .from_utc_datetime(&last_review.naive_utc())
                    .date_naive();
                local_date >= today
            })
        })
        .collect();

    cards_today.sort_by(|a, b| {
        b.memory()
            .last_review_date()
            .cmp(&a.memory().last_review_date())
    });

    cards_today
        .into_iter()
        .map(|sc| {
            let card = sc.card();

            let japanese = card
                .question(lang)
                .map(|q| q.text().to_string())
                .unwrap_or_else(|_| match card {
                    Card::Vocabulary(v) => v.word().text().to_string(),
                    Card::Kanji(k) => k.kanji().text().to_string(),
                    Card::Grammar(g) => g.rule_id().to_string(),
                    Card::Phrase(p) => p.phrase_id().to_string(),
                    Card::Counter(c) => c.suffix().to_string(),
                    Card::Example(e) => e.word().to_string(),
                });

            let meaning = format_answer_text(card, lang);

            let card_type = match CardType::from(card) {
                CardType::Kanji => "kanji",
                CardType::Vocabulary => "vocabulary",
                CardType::Grammar => "grammar",
                CardType::Phrase => "vocabulary",
                CardType::Counter => "counter",
                // Пример (#528 v3) сворачивается в категорию слов —
                // как фразы (владелец слова не различается на дашборде).
                CardType::Example => "vocabulary",
            };

            let (reading, short_description) = match card {
                Card::Kanji(k) => {
                    let on = k.on_readings();
                    let kun = k.kun_readings();
                    let reading = if on.is_empty() && kun.is_empty() {
                        None
                    } else {
                        let parts: Vec<String> = on.iter().chain(kun.iter()).cloned().collect();
                        Some(parts.join("、"))
                    };
                    (reading, None)
                },
                Card::Grammar(g) => {
                    let short_desc = match g.short_description(lang).ok() {
                        Some(CardAnswer::Vocabulary { translations, .. }) => {
                            Some(translations.join(", "))
                        },
                        Some(other) => Some(other.text_projection()),
                        None => None,
                    };
                    (None, short_desc)
                },
                _ => (None, None),
            };

            RecentlyStudiedItem {
                card_id: sc.card_id().to_string(),
                card_type: card_type.to_string(),
                japanese,
                meaning,
                reading,
                short_description,
            }
        })
        .collect()
}

#[derive(Clone, Default)]
pub struct CompletionForecast {
    pub days_remaining: Option<usize>,
    pub is_all_studied: bool,
    pub target_date_label: String,
}

pub fn compute_completion_forecast(
    knowledge_set: &KnowledgeSet,
    history: &[DailyHistoryItem],
    lang: &NativeLanguage,
) -> CompletionForecast {
    use origa::domain::estimate_completion_date;

    let mut new_cards_remaining = 0usize;
    let mut total_cards = 0usize;

    for study_card in knowledge_set.study_cards().values() {
        let card = study_card.card();
        if matches!(CardType::from(card), CardType::Phrase) {
            continue;
        }
        total_cards += 1;
        if study_card.memory().is_new() {
            new_cards_remaining += 1;
        }
    }

    let is_all_studied = total_cards > 0 && new_cards_remaining == 0;

    let (days_remaining, target_date_label) = if new_cards_remaining > 0 {
        match estimate_completion_date(history, new_cards_remaining) {
            Some(date) => {
                let local = chrono::Local.from_utc_datetime(&date.naive_utc());
                let day = local.day();
                let month_idx = (local.month0() as usize).min(11);
                let month_str = match lang {
                    NativeLanguage::Russian => RU_MONTHS_SHORT[month_idx],
                    _ => EN_MONTHS_SHORT[month_idx],
                };
                let date_label = format!("~{} {}", day, month_str);

                let now = chrono::Utc::now();
                let duration = date.signed_duration_since(now);
                let days = (duration.num_hours() as f64 / 24.0).ceil().max(1.0) as usize;

                (Some(days), date_label)
            },
            None => (None, String::new()),
        }
    } else {
        (None, String::new())
    };

    CompletionForecast {
        days_remaining,
        is_all_studied,
        target_date_label,
    }
}

/// Типы карт, которые считает счётчик повторений: слова, кандзи, грамматика
/// и счётные суффиксы. Фразы и примеры исключены — у них собственные
/// пайплайны повторения (issue #809).
///
/// Разбор исчерпывающий: новый тип карты в домене заставит дописать руку
/// здесь, а не молча выпасть из счётчика (прецедент `Card::Example`, #528 v3).
fn is_counted_card_type(card_type: CardType) -> bool {
    match card_type {
        CardType::Vocabulary | CardType::Kanji | CardType::Grammar | CardType::Counter => true,
        CardType::Phrase | CardType::Example => false,
    }
}

/// Сколько карт ждут повторения: срок повторения уже наступил.
///
/// Предикат — доменный [`MemoryHistory::is_due`]: наступил срок, а статус
/// карты не важен — помеченные «Уже знаю» тоже считаются (решение
/// пользователя, issue #809). Новые карты не считаются: срока у них ещё нет.
pub fn compute_due_count(knowledge_set: &KnowledgeSet) -> usize {
    knowledge_set
        .study_cards()
        .values()
        .filter(|study_card| is_counted_card_type(CardType::from(study_card.card())))
        .filter(|study_card| study_card.memory().is_due())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{
        Card, CounterCard, ExampleCard, KnowledgeSet, NativeLanguage, PhraseCard, Question,
        RateMode, Rating, RatingContext, User, VocabularyCard,
    };
    use ulid::Ulid;

    fn set_with_new_cards(n: usize) -> KnowledgeSet {
        let mut set = KnowledgeSet::new();
        for i in 0..n {
            let vocab = VocabularyCard::new_with_pos(
                Question::new(format!("word{i}")).unwrap(),
                None,
                None,
            );
            let _ = set.create_card(Card::Vocabulary(vocab));
        }
        set
    }

    #[test]
    fn completion_forecast_empty_set_is_not_all_studied() {
        let set = KnowledgeSet::new();
        let forecast = compute_completion_forecast(&set, &[], &NativeLanguage::English);
        assert!(!forecast.is_all_studied, "no cards → not all studied");
        assert_eq!(forecast.target_date_label, "");
    }

    #[test]
    fn completion_forecast_new_cards_remaining_is_not_all_studied() {
        let set = set_with_new_cards(3);
        let forecast = compute_completion_forecast(&set, &[], &NativeLanguage::English);
        assert!(!forecast.is_all_studied, "new cards remain → not done");
        // Without history the completion date is not estimable.
        assert_eq!(forecast.days_remaining, None);
    }

    #[test]
    fn completion_forecast_all_cards_reviewed_is_all_studied() {
        let mut set = set_with_new_cards(2);
        // Mark every card as known — no new cards remain.
        for (id, _) in set.study_cards().clone() {
            let _ = set.mark_card_as_known(id);
        }
        let forecast = compute_completion_forecast(&set, &[], &NativeLanguage::English);
        assert!(forecast.is_all_studied, "everything reviewed → done");
    }

    #[test]
    fn studied_today_excludes_phrase_cards_and_old_reviews() {
        let set = set_with_new_cards(2);
        // No review dates → nothing studied today.
        let items = compute_studied_today(&set, &NativeLanguage::English);
        assert!(
            items.is_empty(),
            "cards without a review date are not 'studied today'"
        );
    }

    // ─── Счётчик повторений (issue #809) ───

    fn user_with_card(card: Card) -> User {
        let mut user = User::new(
            "tester@origa.local".to_string(),
            NativeLanguage::English,
            None,
        );
        user.create_card(card).expect("card must be created");
        user
    }

    fn vocabulary_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new_with_pos(
            Question::new(word.to_string()).unwrap(),
            None,
            None,
        ))
    }

    /// Карта со сроком повторения в прошлом: «Уже знаю» ставит срок вчера.
    fn known_card(user: &mut User, card_id: ulid::Ulid) {
        user.mark_card_as_known(card_id)
            .expect("card must be marked known");
    }

    #[test]
    fn due_count_of_empty_set_is_zero() {
        let set = KnowledgeSet::new();
        assert_eq!(compute_due_count(&set), 0);
    }

    #[test]
    fn due_count_skips_cards_without_review_date() {
        // Arrange: новая карта — срока повторения ещё нет.
        let user = user_with_card(vocabulary_card("猫"));

        // Act
        let count = compute_due_count(user.knowledge_set());

        // Assert
        assert_eq!(count, 0, "a new card has no review date yet");
    }

    #[test]
    fn due_count_skips_cards_with_future_review_date() {
        // Arrange: карта, изученная в уроке, — следующий срок в будущем.
        let mut user = user_with_card(vocabulary_card("犬"));
        let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
        user.rate_card(
            card_id,
            Rating::Good,
            RateMode::StandardLesson,
            RatingContext::Explicit,
        )
        .expect("card must be rated");

        // Act
        let count = compute_due_count(user.knowledge_set());

        // Assert
        assert_eq!(count, 0, "a card scheduled in the future is not due");
    }

    #[test]
    fn due_count_counts_known_cards_with_passed_review_date() {
        // Arrange: «Уже знаю» ставит срок вчера — карта просрочена.
        let mut user = user_with_card(vocabulary_card("鳥"));
        let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
        known_card(&mut user, card_id);

        // Act
        let count = compute_due_count(user.knowledge_set());

        // Assert: статус не важен (решение пользователя, issue #809) —
        // считаем и карты, помеченные «Уже знаю».
        assert_eq!(count, 1, "a known card past its review date is due");
    }

    #[test]
    fn due_count_mixes_passed_and_future_cards() {
        // Arrange: два просроченных, одна новая, одна с будущим сроком.
        let mut user = User::new(
            "tester@origa.local".to_string(),
            NativeLanguage::English,
            None,
        );
        let mut created = Vec::new();
        for word in ["猫", "犬", "鳥", "魚"] {
            created.push(
                user.create_card(vocabulary_card(word))
                    .expect("card created"),
            );
        }
        known_card(&mut user, *created[0].card_id());
        known_card(&mut user, *created[1].card_id());
        user.rate_card(
            *created[3].card_id(),
            Rating::Good,
            RateMode::StandardLesson,
            RatingContext::Explicit,
        )
        .expect("card must be rated");

        // Act
        let count = compute_due_count(user.knowledge_set());

        // Assert
        assert_eq!(count, 2, "only the two cards past their date count");
    }

    #[test]
    fn due_count_skips_phrase_and_example_cards() {
        // Arrange: фраза и пример с просроченным сроком — свои пайплайны.
        let mut user = User::new(
            "tester@origa.local".to_string(),
            NativeLanguage::English,
            None,
        );
        let phrase = user
            .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
            .expect("phrase created");
        let example = user
            .create_card(Card::Example(ExampleCard::new("猫", 1)))
            .expect("example created");
        known_card(&mut user, *phrase.card_id());
        known_card(&mut user, *example.card_id());

        // Act
        let count = compute_due_count(user.knowledge_set());

        // Assert
        assert_eq!(count, 0, "phrases and examples are not counted");
    }

    #[test]
    fn due_count_counts_counter_suffix_cards() {
        // Arrange: счётный суффикс — считаемый тип карты.
        let mut user = user_with_card(Card::Counter(CounterCard::new("つ")));
        let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
        known_card(&mut user, card_id);

        // Act
        let count = compute_due_count(user.knowledge_set());

        // Assert
        assert_eq!(count, 1, "counter suffixes are counted");
    }

    #[test]
    fn due_count_never_exceeds_overview_total() {
        // Arrange: две словарные карты с прошедшим сроком и одна фраза с
        // прошедшим сроком. Фраза не считается ни счётчиком, ни сводкой —
        // если она попадёт в счётчик, число (3) превысит «всего» (2).
        let mut user = User::new(
            "tester@origa.local".to_string(),
            NativeLanguage::English,
            None,
        );
        let first = user
            .create_card(vocabulary_card("猫"))
            .expect("card created");
        let second = user
            .create_card(vocabulary_card("犬"))
            .expect("card created");
        let phrase = user
            .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
            .expect("phrase created");
        known_card(&mut user, *first.card_id());
        known_card(&mut user, *second.card_id());
        known_card(&mut user, *phrase.card_id());

        // Act
        let due = compute_due_count(user.knowledge_set());
        let total = origa::domain::compute_today_overview(user.knowledge_set(), &[]).total();

        // Assert: счётчик — подмножество «всего» в шапке виджета: каждая
        // не-фразовая карта попадает ровно в один бакет сводки, а due ⇒
        // «не новая» ⇒ бакет один из трёх «изученных».
        assert_eq!(due, 2, "the phrase card is not counted");
        assert!(
            due <= total,
            "due count {due} must not exceed the overview total {total}"
        );
    }
}
