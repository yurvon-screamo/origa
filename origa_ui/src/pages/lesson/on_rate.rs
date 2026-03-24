use super::lesson_state::LessonContext;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, RateMode, Rating};
use origa::traits::UserRepository;
use origa::use_cases::{CreateGrammarCardUseCase, RateCardUseCase};
use tracing::warn;
use ulid::Ulid;

pub fn create_on_rate_callback(
    lesson_state: RwSignal<super::lesson_state::LessonState>,
    lesson_ctx: LessonContext,
    is_rating: RwSignal<Option<Ulid>>,
) -> Callback<Rating> {
    let is_disposed = use_context::<StoredValue<bool>>().expect("is_disposed must be provided");

    Callback::new(move |rating: Rating| {
        let state = lesson_state.get_untracked();

        if let Some(card_id) = state.card_ids.get(state.current_index) {
            let card_id = *card_id;
            is_rating.set(Some(card_id));
            let repo = lesson_ctx.repository.clone();
            let lesson_state = lesson_state;
            let is_completed = lesson_ctx.is_completed;
            let is_rating = is_rating;

            spawn_local(async move {
                let use_case = RateCardUseCase::new(&repo);

                if let Err(e) = use_case
                    .execute(card_id, RateMode::StandardLesson, rating)
                    .await
                {
                    warn!(error = ?e, "Failed to rate card");
                }

                let state_snapshot = lesson_state.get_untracked();

                if let Some(card_view) = state_snapshot.cards.get(&card_id)
                    && let origa::domain::LessonCardView::GrammarMutated { grammar_info, .. } =
                        card_view
                    && let Some(grammar_rule_id) = grammar_info.rule_id()
                {
                    let grammar_use_case = RateCardUseCase::new(&repo);

                    if let Some(user) = repo.get_current_user().await.ok().flatten() {
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
                            if let Err(e) = grammar_use_case
                                .execute(grammar_card_id, RateMode::StandardLesson, rating)
                                .await
                            {
                                warn!(error = ?e, "Failed to rate grammar card during dual rating");
                            }
                        } else {
                            let create_use_case = CreateGrammarCardUseCase::new(&repo);
                            let cards = create_use_case.execute(vec![grammar_rule_id]).await;

                            if let Ok(grammar_cards) = cards
                                && let Some(grammar_card) = grammar_cards.first()
                                && let Err(e) = grammar_use_case
                                    .execute(
                                        *grammar_card.card_id(),
                                        RateMode::StandardLesson,
                                        rating,
                                    )
                                    .await
                            {
                                warn!(error = ?e, "Failed to rate newly created grammar card during dual rating");
                            }
                        }
                    }
                }

                if is_disposed.get_value() {
                    return;
                }

                lesson_state.update(|state| {
                    let next_index = state.current_index + 1;
                    let total = state.card_ids.len();

                    state.review_count += 1;

                    if next_index >= total {
                        is_completed.set(true);
                    } else {
                        state.current_index = next_index;
                        state.showing_answer = false;
                        state.selected_quiz_option = None;
                        state.selected_yesno_answer = None;
                    }
                });

                is_rating.set(None);
            });
        }
    })
}
