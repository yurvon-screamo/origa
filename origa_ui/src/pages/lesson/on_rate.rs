use super::lesson_state::LessonContext;
use crate::hooks::phrase_checker;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, CardType, LessonCard, RateMode, Rating};
use origa::traits::UserRepository;
use origa::use_cases::{CreatePhraseCardUseCase, RateCardWithSideEffectsUseCase};
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
            let use_case = RateCardWithSideEffectsUseCase::new(&repo);

            if let Err(e) = use_case
                .execute(card_id, rate_mode, rating, grammar_rule_id)
                .await
            {
                warn!(error = ?e, "Failed to rate card");
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
