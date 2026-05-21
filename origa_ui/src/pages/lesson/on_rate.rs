use super::lesson_state::LessonContext;
use crate::hooks::phrase_checker;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, CardType, LessonCard, RateMode, Rating};
use origa::traits::UserRepository;
use origa::use_cases::{CreateGrammarCardUseCase, CreatePhraseCardUseCase, RateCardUseCase};
use tracing::warn;
use ulid::Ulid;

fn determine_rate_mode(card: &LessonCard, current_index: usize, core_count: usize) -> RateMode {
    let is_phrase = current_index >= core_count;
    if is_phrase {
        RateMode::PhraseReview
    } else if card.is_short_term() {
        RateMode::ShortTerm
    } else {
        match CardType::from(card.card()) {
            CardType::Grammar => RateMode::GrammarReview,
            CardType::Kanji => RateMode::KanjiReview,
            CardType::Vocabulary | CardType::Phrase => RateMode::StandardLesson,
        }
    }
}

fn extract_grammar_rule_id(card: &LessonCard) -> Option<Ulid> {
    match card.view() {
        origa::domain::LessonCardView::GrammarMutated { grammar_info, .. } => {
            grammar_info.rule_id()
        },
        origa::domain::LessonCardView::GrammarQuiz(gq) => gq.grammar_info().rule_id(),
        _ => None,
    }
}

async fn handle_grammar_dual_rating<R: UserRepository>(
    grammar_rule_id: Ulid,
    repo: &R,
    rating: Rating,
) {
    let rate_use_case = RateCardUseCase::new(repo);
    let Some(user) = repo.get_current_user().await.ok().flatten() else {
        return;
    };

    let grammar_card_id = user
        .knowledge_set()
        .study_cards()
        .iter()
        .find(|(_, study_card)| {
            if let Card::Grammar(grammar_card) = study_card.card() {
                grammar_card.rule_id() == &grammar_rule_id
            } else {
                false
            }
        })
        .map(|(id, _)| *id);

    if let Some(grammar_card_id) = grammar_card_id {
        if let Err(e) = rate_use_case
            .execute(grammar_card_id, RateMode::GrammarReview, rating)
            .await
        {
            warn!(error = ?e, "Failed to rate grammar card during dual rating");
        }
        return;
    }

    let create_use_case = CreateGrammarCardUseCase::new(repo);
    let cards = create_use_case.execute(vec![grammar_rule_id]).await;

    if let Ok(grammar_cards) = cards
        && let Some(grammar_card) = grammar_cards.first()
        && let Err(e) = rate_use_case
            .execute(*grammar_card.card_id(), RateMode::GrammarReview, rating)
            .await
    {
        warn!(error = ?e, "Failed to rate newly created grammar card during dual rating");
    }
}

async fn check_and_create_ready_phrases<R: UserRepository>(
    card_id: Ulid,
    repo: &R,
    rating: Rating,
) {
    if rating == Rating::Again {
        return;
    }

    let Some(user) = repo.get_current_user().await.ok().flatten() else {
        return;
    };

    let Some(sc) = user.knowledge_set().study_cards().get(&card_id) else {
        return;
    };

    let Card::Vocabulary(vocab) = sc.card() else {
        return;
    };

    let word = vocab.word().text().to_string();
    let ready_phrases =
        phrase_checker::find_ready_phrases(&word, user.knowledge_set().study_cards());

    if !ready_phrases.is_empty() {
        let create_phrase_use_case = CreatePhraseCardUseCase::new(repo);
        if let Err(e) = create_phrase_use_case.execute(ready_phrases).await {
            warn!(error = ?e, "Failed to create phrase cards");
        }
    }
}

fn advance_lesson_state(
    lesson_state: RwSignal<super::lesson_state::LessonState>,
    is_completed: RwSignal<bool>,
) {
    lesson_state.update(|state| {
        let next_index = state.current_index + 1;
        let total = state.card_ids.len();

        state.review_count += 1;
        state.waiting_for_next = false;
        state.pending_rating = None;

        if next_index >= total {
            is_completed.set(true);
        } else {
            state.current_index = next_index;
            state.showing_answer = false;
            state.selected_quiz_option = None;
            state.selected_yesno_answer = None;
            state.dont_know_selected = false;
            state.selected_quiz_options.clear();
            state.multi_quiz_submitted = false;
            state.multi_result = None;
        }
    });
}

pub fn create_on_rate_callback(
    lesson_state: RwSignal<super::lesson_state::LessonState>,
    lesson_ctx: LessonContext,
    is_rating: RwSignal<Option<Ulid>>,
) -> Callback<Rating> {
    let Some(is_disposed) = use_context::<StoredValue<()>>() else {
        return Callback::new(move |_: Rating| {});
    };

    Callback::new(move |rating: Rating| {
        let state = lesson_state.get_untracked();

        let Some(card_id) = state.card_ids.get(state.current_index) else {
            return;
        };

        let card_id = *card_id;
        is_rating.set(Some(card_id));

        let repo = lesson_ctx.repository.clone();
        let lesson_state = lesson_state;
        let is_completed = lesson_ctx.is_completed;
        let is_rating = is_rating;

        let rate_mode = state
            .cards
            .get(&card_id)
            .map(|c| determine_rate_mode(c, state.current_index, state.core_count))
            .unwrap_or(RateMode::StandardLesson);

        let grammar_rule_id = state.cards.get(&card_id).and_then(extract_grammar_rule_id);

        spawn_local(async move {
            let use_case = RateCardUseCase::new(&repo);

            if let Err(e) = use_case.execute(card_id, rate_mode, rating).await {
                warn!(error = ?e, "Failed to rate card");
            }

            if let Some(grammar_rule_id) = grammar_rule_id {
                handle_grammar_dual_rating(grammar_rule_id, &repo, rating).await;
            }

            check_and_create_ready_phrases(card_id, &repo, rating).await;

            if is_disposed.is_disposed() {
                return;
            }

            advance_lesson_state(lesson_state, is_completed);
            is_rating.set(None);
        });
    })
}
