use std::collections::HashSet;

use crate::i18n::use_i18n;
use crate::ui_components::{Modal, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::dictionary::grammar::{FormatAction, FormatActionGroup};
use origa::domain::{
    Card as DomainCard, GrammarRule, NativeLanguage, PartOfSpeech, User, apply_format_actions,
};
use rand::prelude::{IndexedRandom, SliceRandom};

#[derive(Clone, Copy, PartialEq)]
enum QuizState {
    Idle,
    Answered { selected: usize, correct: bool },
}

fn find_matching_vocab(user: &User, pos: &PartOfSpeech) -> Vec<String> {
    user.knowledge_set()
        .study_cards()
        .values()
        .filter(|sc| sc.memory().is_known_card() || sc.memory().is_in_progress())
        .filter_map(|sc| match sc.card() {
            DomainCard::Vocabulary(v) => {
                let word = v.word().text().to_string();
                let vocab_pos = v.part_of_speech().ok()?;
                if vocab_pos == *pos { Some(word) } else { None }
            },
            _ => None,
        })
        .collect()
}

fn generate_question(rule: &GrammarRule, user: &User) -> Option<(String, Vec<String>, usize)> {
    let supported_pos = rule.apply_to();
    let pos = supported_pos.first()?.clone();

    let matching_vocab = find_matching_vocab(user, &pos);
    if matching_vocab.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    let word = matching_vocab.choose(&mut rng)?.clone();

    let correct = rule.format(&word, &pos).ok()?;
    let actions = rule.format_actions_for_pos(&pos)?;

    let distractors = generate_distractors(actions, &word, &pos, &correct, &mut rng);

    let mut options: Vec<String> = distractors;
    options.push(correct.clone());
    options.shuffle(&mut rng);
    let correct_index = options.iter().position(|o| *o == correct)?;

    Some((word, options, correct_index))
}

fn generate_distractors(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    correct_text: &str,
    rng: &mut impl rand::Rng,
) -> Vec<String> {
    let mut distractors = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(correct_text.to_string());

    for _ in 0..30 {
        if distractors.len() >= 3 {
            break;
        }
        if let Some(d) = apply_mutated_pattern(rules, source_word, pos, rng) {
            if !seen.contains(&d) {
                seen.insert(d.clone());
                distractors.push(d);
            }
        }
    }

    distractors
}

fn apply_mutated_pattern(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    rng: &mut impl rand::Rng,
) -> Option<String> {
    let mutable_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, a)| a.group() != FormatActionGroup::Universal)
        .map(|(i, _)| i)
        .collect();

    if mutable_indices.is_empty() {
        return None;
    }

    let idx = mutable_indices.choose(rng)?;
    let original_action = &rules[*idx];
    let alternatives = original_action.mutation_alternatives();
    if alternatives.is_empty() {
        return None;
    }

    let alternative = alternatives.choose(rng)?;
    let mut mutated: Vec<FormatAction> = rules.to_vec();
    mutated[*idx] = (*alternative).clone();

    apply_format_actions(source_word, &mutated, pos).ok()
}

#[component]
pub fn GrammarQuizModal(
    rule: &'static GrammarRule,
    #[prop(into)] native_language: Signal<NativeLanguage>,
    user: User,
    #[prop(into)] is_open: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();

    let is_open_rw = RwSignal::new(is_open.get_untracked());
    Effect::new(move || {
        is_open_rw.set(is_open.get());
    });

    let question_number = RwSignal::new(1u32);
    let quiz_state: RwSignal<QuizState> = RwSignal::new(QuizState::Idle);

    let initial = generate_question(rule, &user);
    let quiz_word: RwSignal<String> = RwSignal::new(
        initial
            .as_ref()
            .map(|(w, _, _)| w.clone())
            .unwrap_or_default(),
    );
    let quiz_options: RwSignal<Vec<String>> = RwSignal::new(
        initial
            .as_ref()
            .map(|(_, o, _)| o.clone())
            .unwrap_or_default(),
    );
    let quiz_correct: RwSignal<usize> =
        RwSignal::new(initial.as_ref().map(|(_, _, c)| *c).unwrap_or(0));
    let has_question: RwSignal<bool> = RwSignal::new(initial.is_some());

    let title_text = Signal::derive({
        let lang = native_language.get_untracked();
        let title = rule.content(&lang).title().to_string();
        move || title.clone()
    });

    let description_signal = Signal::derive({
        let lang = native_language.get_untracked();
        let desc = rule.content(&lang).short_description().to_string();
        move || desc.clone()
    });

    let on_next = Callback::new(move |_| {
        if let Some((word, options, correct)) = generate_question(rule, &user) {
            quiz_word.set(word);
            quiz_options.set(options);
            quiz_correct.set(correct);
            has_question.set(true);
            question_number.update(|n| *n += 1);
        } else {
            has_question.set(false);
        }
        quiz_state.set(QuizState::Idle);
    });

    let option_0 = Memo::new(move |_| quiz_options.get().first().cloned());
    let option_1 = Memo::new(move |_| quiz_options.get().get(1).cloned());
    let option_2 = Memo::new(move |_| quiz_options.get().get(2).cloned());
    let option_3 = Memo::new(move |_| quiz_options.get().get(3).cloned());

    let option_class = move |index: usize| -> String {
        let state = quiz_state.get();
        let correct = quiz_correct.get();
        let is_correct = index == correct;
        let base = "p-3 border text-left transition-all";

        match state {
            QuizState::Idle => {
                format!(
                    "{} cursor-pointer quiz-option-neutral hover:opacity-80",
                    base
                )
            },
            QuizState::Answered { selected, .. } => {
                let ptr = "pointer-events-none";
                if index == selected && is_correct {
                    format!("{} {} quiz-option-correct", base, ptr)
                } else if index == selected {
                    format!("{} {} quiz-option-wrong", base, ptr)
                } else if is_correct {
                    format!("{} {} quiz-option-correct", base, ptr)
                } else {
                    format!("{} {} quiz-option-dimmed", base, ptr)
                }
            },
        }
    };

    let on_option_click = move |index: usize| {
        if matches!(quiz_state.get(), QuizState::Idle) {
            let correct = index == quiz_correct.get();
            quiz_state.set(QuizState::Answered {
                selected: index,
                correct,
            });
        }
    };

    let no_words_signal = Signal::derive({
        let text = i18n
            .get_keys()
            .grammar_page()
            .quiz_no_words()
            .inner()
            .to_string();
        move || text.clone()
    });

    let next_signal = Signal::derive({
        let text = i18n
            .get_keys()
            .grammar_page()
            .quiz_next()
            .inner()
            .to_string();
        move || text.clone()
    });

    let close_signal = Signal::derive({
        let text = i18n
            .get_keys()
            .grammar_page()
            .quiz_close()
            .inner()
            .to_string();
        move || text.clone()
    });

    let correct_label = i18n
        .get_keys()
        .grammar_page()
        .quiz_correct()
        .inner()
        .to_string();
    let wrong_label = i18n
        .get_keys()
        .grammar_page()
        .quiz_wrong()
        .inner()
        .to_string();

    let result_text = Memo::new(move |_| match quiz_state.get() {
        QuizState::Answered { correct: true, .. } => correct_label.clone(),
        QuizState::Answered { correct: false, .. } => wrong_label.clone(),
        _ => String::new(),
    });

    let result_class = Memo::new(move |_| match quiz_state.get() {
        QuizState::Answered { correct: true, .. } => "text-[var(--success)] font-bold".to_string(),
        QuizState::Answered { correct: false, .. } => "text-[var(--error)] font-bold".to_string(),
        _ => String::new(),
    });

    let is_answered = Memo::new(move |_| matches!(quiz_state.get(), QuizState::Answered { .. }));

    view! {
        <Modal
            test_id=Signal::derive(|| "grammar-quiz-modal".to_string())
            is_open=is_open_rw
            title=title_text
        >
            <div class="p-4 space-y-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {description_signal}
                </Text>

                <Show when=move || !has_question.get()>
                    <div class="text-center py-8">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            {no_words_signal}
                        </Text>
                        <div class="mt-4">
                            <button
                                class="px-4 py-2 bg-[var(--accent-olive)] text-white rounded hover:opacity-80 transition-opacity cursor-pointer"
                                on:click=move |_| on_close.run(())
                            >
                                {close_signal}
                            </button>
                        </div>
                    </div>
                </Show>

                <Show when=move || has_question.get()>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {move || format!("#{}", question_number.get())}
                    </Text>

                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                        {move || {
                            i18n.get_keys().grammar_page().quiz_question().inner().to_string()
                                .replacen("{}", &quiz_word.get(), 1)
                        }}
                    </Text>

                    <div class="grid grid-cols-2 gap-2">
                        <button
                            class=move || option_class(0)
                            on:click=move |_| on_option_click(0)
                        >
                            <Text size=TextSize::Default>
                                {move || option_0.get().unwrap_or_default()}
                            </Text>
                        </button>
                        <button
                            class=move || option_class(1)
                            on:click=move |_| on_option_click(1)
                        >
                            <Text size=TextSize::Default>
                                {move || option_1.get().unwrap_or_default()}
                            </Text>
                        </button>
                        <button
                            class=move || option_class(2)
                            on:click=move |_| on_option_click(2)
                        >
                            <Text size=TextSize::Default>
                                {move || option_2.get().unwrap_or_default()}
                            </Text>
                        </button>
                        <button
                            class=move || option_class(3)
                            on:click=move |_| on_option_click(3)
                        >
                            <Text size=TextSize::Default>
                                {move || option_3.get().unwrap_or_default()}
                            </Text>
                        </button>
                    </div>

                    <Show when=move || is_answered.get()>
                        <div class="text-center">
                            <Text
                                size=TextSize::Default
                                class=Signal::derive(move || result_class.get())
                            >
                                {move || result_text.get()}
                            </Text>
                        </div>

                        <div class="flex justify-center gap-3 mt-4">
                            <button
                                class="px-4 py-2 bg-[var(--accent-olive)] text-white rounded hover:opacity-80 transition-opacity cursor-pointer"
                                on:click=move |_| on_next.run(())
                            >
                                {next_signal}
                            </button>
                            <button
                                class="px-4 py-2 border rounded hover:opacity-80 transition-opacity cursor-pointer"
                                on:click=move |_| on_close.run(())
                            >
                                {close_signal}
                            </button>
                        </div>
                    </Show>
                </Show>
            </div>
        </Modal>
    }
}
