use crate::i18n::*;
use crate::ui_components::{
    AudioButtons, MarkdownText, MarkdownVariant, Tag, TagVariant, Text, TextSize,
    TypographyVariant, speak_word, stop_current_audio, word_audio_available,
};
use leptos::prelude::*;
use leptos_icons::Icon;
use origa::domain::{Card as DomainCard, NativeLanguage};
use std::collections::HashSet;

use super::answer_display::{CardAnswerDisplay, extract_card_answer};
use super::binary_rating_buttons::BinaryRatingButtons;
use super::card_text::{card_question_text, display_question_text};
use super::card_type::CardType;
use super::lesson_card_header::LessonTagsRow;
use super::lesson_card_shell::LessonCardShell;
use super::lesson_state::LessonContext;
use super::reveal_answer_button::RevealAnswerButton;

/// Audio-recall card: the word is NEVER shown on the question side — the
/// learner hears it (autoplay + manual replay) and opens the answer with
/// «Показать». The answer side is the classic one: the word, its
/// translation and the binary self-assessment («Не знаю» / «Знаю») which
/// rates and advances immediately, like the normal cards' rating buttons
/// (ADR-050). Manual replay is NOT gated by the lesson mute: it is an
/// explicit user action, and a textless card without sound would be
/// unanswerable (see docs — AudioRecall mute matrix).
#[component]
pub fn AudioRecallCardView(
    card: DomainCard,
    show_result: Signal<bool>,
    on_answer: Callback<bool>,
    on_reveal: Callback<()>,
    on_replay: Callback<()>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(default = Signal::derive(|| false))] disabled: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card_type = CardType::from(&card);
    let lang = native_language;

    let question_text = card_question_text(&card, &lang, card_type);
    let display_word = display_question_text(&card, &lang, card_type);
    let word_stored = StoredValue::new(display_word);
    // The tags-row replay button speaks the raw word (no な-suffix) — the
    // same text the question-side big button replays via `on_replay`.
    let header_replay_text = StoredValue::new(question_text.clone());

    let answer_data = extract_card_answer(&card, &lang, &card_type);
    let answer_translations = StoredValue::new(answer_data.translations);
    let answer_description = StoredValue::new(answer_data.description);
    let answer_text = StoredValue::new(answer_data.text);

    let lesson_ctx = use_context::<LessonContext>();
    // Autoplay once when the question side mounts. Guards mirror the
    // availability predicate (pitch audio OR TTS) plus the lesson mute —
    // a muted user never reaches this component in the first place (mode
    // sampling), but the guard keeps the invariant if state drifts.
    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        let answered = show_result.get();
        if !answered && !is_muted && word_audio_available(&question_text) {
            speak_word(&question_text, 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let tags_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
            <LessonTagsRow>
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label(&i18n)}
                </Tag>
                <Tag variant=Signal::derive(move || TagVariant::Filled)>
                    {t!(i18n, lesson.audio_tag)}
                </Tag>
                // Answer side: a compact replay button on the right edge of
                // the tags row, like the normal cards' header audio — the
                // question side keeps the big play button (its core
                // interaction).
                <Show when=move || show_result.get()>
                    <div class="ml-auto shrink-0">
                        <AudioButtons
                            text=header_replay_text.get_value()
                            audio_path=None
                            test_id=Signal::derive(|| "audio-recall-header-play-btn".to_string())
                        />
                    </div>
                </Show>
            </LessonTagsRow>
        }
        .into_any()
    });

    view! {
        <LessonCardShell
            tags=tags_row
            test_id="audio-recall-card-root"
        >
        <Show when=move || !show_result.get()>
                <div class="text-center mb-3 sm:mb-6">
                    <button
                        data-testid="audio-recall-play-btn"
                        class="audio-player-btn p-3 sm:p-4 rounded-full border transition-all cursor-pointer hover:bg-[var(--bg-hover)]"
                        on:click=move |_| on_replay.run(())
                    >
                        <Icon icon=icondata::LuVolume2 width="1.5em" height="1.5em" />
                    </button>
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        {t!(i18n, lesson.listen_word)}
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted class="mt-1">
                        <span class="kbd-hint">{t!(i18n, lesson.enter_key)}</span>
                    </Text>
                </div>

                // Question side: audio + the classic «Показать» reveal —
                // the self-assessment lives on the ANSWER side, after the
                // learner has seen the word and its translation. Space
                // follows the universal reveal pattern shared with every
                // other card type — the replay moved to Enter.
                <div class="flex justify-center">
                    <RevealAnswerButton
                        test_id="audio-recall-reveal-btn"
                        on_click=on_reveal
                    />
                </div>
            </Show>

            <Show when=move || show_result.get()>
                // The word itself is part of the ANSWER side — the
                // learner checks what they heard against the written form.
                <div class="text-center mb-2">
                    <MarkdownText
                        content=Signal::derive(move || word_stored.get_value())
                        known_kanji=known_kanji.get()
                        variant=Signal::derive(|| MarkdownVariant::Large)
                    />
                </div>

                <CardAnswerDisplay
                    translations=Signal::derive(move || answer_translations.get_value())
                    description=Signal::derive(move || answer_description.get_value())
                    text=Signal::derive(move || answer_text.get_value())
                    known_kanji=known_kanji
                />

                // Self-assessment rates and advances immediately
                // (ADR-050): the word and translation are already on
                // screen, there is nothing else to hold the learner for.
                <BinaryRatingButtons
                    left_label=Signal::derive(move || t_string!(i18n, lesson.dont_know_rating).to_string())
                    right_label=Signal::derive(move || t_string!(i18n, lesson.know).to_string())
                    left_test_id="audio-recall-dont-know-btn"
                    right_test_id="audio-recall-know-btn"
                    left_hint="[1]"
                    right_hint="[2]"
                    disabled=disabled
                    on_left=Callback::new(move |_| on_answer.run(false))
                    on_right=Callback::new(move |_| on_answer.run(true))
                />
            </Show>
        </LessonCardShell>
    }
}
