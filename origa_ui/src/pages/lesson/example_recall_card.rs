<<<<<<< HEAD
//! Example-recall companion (#528): the question side shows a textbook
//! sentence using the word — no translation; Space (or the button)
//! reveals the translation FIRST, and only then the learner rates
//! understood / didn't understand (the answer is visible before the
//! self-assessment, same order as every other card). The rating never
//! touches FSRS: advancing uses a rating-free lesson-state transition.
//!
//! Visual etalon: the phrase card — the sentence renders through the
//! token translator (`TranslatorText`), the answer side repeats it above
//! a divider with the translation under it; the audio button sits in the
//! tags row above the card (top-right), the binary rating buttons render
//! BELOW the card exactly like `RatingButtons`.
=======
//! Example-recall view (#528): the question side shows a textbook sentence
//! using the word — no translation; the learner reveals the translation
//! first (the answer MUST be visible before the self-assessment, same
//! order as every other card), then rates understood / didn't understand;
//! the rating buttons advance the card. The self-assessment never touches
//! FSRS: advancing uses a rating-free lesson-state transition.
>>>>>>> origin/master

use leptos::prelude::*;
use leptos::task::spawn_local;

use super::LESSON_CARD_CLASS;
<<<<<<< HEAD
use crate::i18n::{t, use_i18n};
=======
use crate::i18n::use_i18n;
>>>>>>> origin/master
use crate::loaders::example_loader::{WordExample, load_example_detail};
use crate::ui_components::{
<<<<<<< HEAD
    AudioButtons, Button, ButtonVariant, Card, Tag, Text, TextSize, TranslatorText,
    TypographyVariant,
=======
    AudioButtons, Button, ButtonVariant, Card, FuriganaText, Text, TextSize, TypographyVariant,
>>>>>>> origin/master
};
use origa::domain::NativeLanguage;

#[component]
pub fn ExampleRecallCard(
    sentence_id: u32,
    show_answer: Signal<bool>,
    on_show_answer: Callback<()>,
    on_advance: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();

    let example: RwSignal<Option<WordExample>> = RwSignal::new(None);
    // A failed chunk load (CDN 404, offline, corrupt file) must not turn
    // the card into a dead end: the fallback renders an explicit
    // «unavailable» notice with the advance button, so the lesson always
    // has a way forward.
    let load_failed: RwSignal<bool> = RwSignal::new(false);
    let tts_spoken = RwSignal::new(false);
    spawn_local(async move {
        match load_example_detail(sentence_id).await {
            Ok(detail) => example.set(Some(WordExample {
                detail,
                start: -1,
                end: -1,
            })),
            Err(_) => load_failed.set(true),
        }
    });

    // Auto-speak the sentence once it renders — same as the question side
<<<<<<< HEAD
    // of every other lesson card. Respects the lesson mute toggle when
    // the lesson context is around (headless mounts have none).
    let lesson_ctx = use_context::<super::LessonContext>();
    let tts_example = example;
    Effect::new(move |_| {
        let Some(we) = tts_example.get() else {
=======
    // of every other lesson card. Respects the lesson mute toggle when the
    // lesson context is around (headless mounts have none).
    let lesson_ctx = use_context::<super::LessonContext>();
    Effect::new(move |_| {
        let Some(we) = example.get() else {
>>>>>>> origin/master
            return;
        };
        if tts_spoken.get() {
            return;
        }
        tts_spoken.set(true);
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !is_muted && crate::ui_components::is_speech_supported() {
            let _ = crate::ui_components::speak_tts_text(&we.detail.text, 1.0);
        }
    });
<<<<<<< HEAD

    view! {
        // Tags row above the card — the shared lesson-card pattern: the
        // card-type tag on the left, the sentence audio button pushed to
        // the top-right corner.
        <div class="flex items-center gap-2 flex-wrap min-w-0 mb-2 px-1">
            <Tag>{t!(i18n, lesson.example_tag)}</Tag>
            <Show when=move || tts_example.get().is_some()>
                <div class="ml-auto">
                    {move || {
                        let we = tts_example.get().expect("checked by Show");
=======

    // Reveal-first pipeline (owner fix): the translation is hidden until
    // the learner asks for it; the understood / didn't-understand choice
    // happens on the ANSWER side, after the answer was seen — otherwise a
    // pre-reveal guess is a coin flip. The rating buttons advance
    // (rating-free), no separate next button.
    let revealed = RwSignal::new(false);

    view! {
        <Card
            class=Signal::derive(|| LESSON_CARD_CLASS.to_string())
            shadow=Signal::derive(|| true)
            test_id=Signal::derive(|| "lesson-example-recall-card".to_string())
        >
            <div class="example-recall" data-testid="lesson-example-recall">
                <div class="example-recall-ja" data-testid="lesson-example-ja">
                    {move || {
                        let Some(we) = example.get() else {
                            return ().into_any();
                        };
                        let word = word.get_value();
                        let (head, mid, tail) = highlight_parts(&we, &word);
                        let known = known_kanji.get();
                        if mid.is_empty() {
                            // No highlight to draw: one run (an empty highlight
                            // span reads as a stray tick mark).
                            return view! {
                                <FuriganaText text=head known_kanji=known/>
                            }
                            .into_any();
                        }
                        view! {
                            <FuriganaText text=head.clone() known_kanji=known.clone()/>
                            <FuriganaText
                                text=mid.clone()
                                known_kanji=known.clone()
                                class=String::from("word-example-highlight")
                            />
                            <FuriganaText text=tail.clone() known_kanji=known/>
                        }
                        .into_any()
                    }}
                </div>

                <Show when=move || example.get().is_some()>
                    {move || {
                        let we = example.get().expect("checked by Show");
>>>>>>> origin/master
                        view! {
                            <AudioButtons
                                text=we.detail.text
                                audio_path=None
                                test_id=Signal::derive(|| "lesson-example-audio".to_string())
                            />
                        }
                    }}
<<<<<<< HEAD
                </div>
            </Show>
        </div>

        <Card
            class=Signal::derive(|| LESSON_CARD_CLASS.to_string())
            shadow=Signal::derive(|| true)
            test_id=Signal::derive(|| "lesson-example-recall-card".to_string())
        >
            <div class="flex-1 flex flex-col justify-center text-center" data-testid="lesson-example-recall">
                {move || {
                    if load_failed.get() {
                        return view! {
                            <div
                                class="example-recall-unavailable"
                                data-testid="lesson-example-unavailable"
                            >
                                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                    {t!(i18n, lesson.example_unavailable)}
                                </Text>
                            </div>
=======
                </Show>
            </div>

            <div class="example-recall-footer">
                {move || {
                    if load_failed.get() {
                        return view! {
                            <div
                                class="example-recall-unavailable"
                                data-testid="lesson-example-unavailable"
                            >
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    {i18n.get_keys().lesson().example_unavailable().inner().to_string()}
                                </Text>
                            </div>
                            <Button
                                variant=ButtonVariant::Olive
                                on_click=Callback::new(move |_| on_advance.run(()))
                                test_id="lesson-example-next"
                            >
                                {crate::i18n::t!(i18n, lesson.next)}
                            </Button>
                        }
                        .into_any();
                    }
                    let Some(we) = example.get() else {
                        return ().into_any();
                    };
                    let translation = we
                        .detail
                        .translation(&native_language.get())
                        .unwrap_or_default()
                        .to_string();

                    if !revealed.get() {
                        return view! {
                            <Button
                                variant=ButtonVariant::Olive
                                on_click=Callback::new(move |_| revealed.set(true))
                                test_id="lesson-example-reveal"
                            >
                                {i18n
                                    .get_keys()
                                    .lesson()
                                    .example_reveal_translation()
                                    .inner()
                                    .to_string()}
                            </Button>
>>>>>>> origin/master
                        }
                        .into_any();
                    }
                    let Some(we) = example.get() else {
                        return ().into_any();
                    };
                    let translation = we
                        .detail
                        .translation(&lesson_language())
                        .unwrap_or_default()
                        .to_string();

                    if !show_answer.get() {
                        // Question side: the sentence alone (token
                        // translator, no furigana — the phrase-card etalon).
                        return view! {
                            <div class="example-recall-ja" data-testid="lesson-example-ja">
                                <TranslatorText
                                    text=we.detail.text.clone()
                                    class=Signal::derive(|| "text-2xl leading-relaxed".to_string())
                                    test_id=Signal::derive(|| "lesson-example-sentence".to_string())
                                />
                            </div>
                        }
                        .into_any();
                    }
                    // Answer side: the sentence again + divider + the
                    // translation (the LessonCardAnswer phrase etalon).
                    view! {
<<<<<<< HEAD
                        <div class="example-recall-ja" data-testid="lesson-example-ja">
                            <TranslatorText
                                text=we.detail.text.clone()
                                class=Signal::derive(|| "text-2xl leading-relaxed".to_string())
                                test_id=Signal::derive(|| "lesson-example-sentence".to_string())
                            />
                            <div class="border-t border-[var(--border-light)] pt-4 mt-4">
                                <div class="max-w-max mx-auto">
                                    <Text size=TextSize::Large variant=TypographyVariant::Primary>
                                        {translation}
                                    </Text>
                                </div>
                            </div>
=======
                        <div
                            class="example-recall-translation"
                            data-testid="lesson-example-translation"
                        >
                            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                {translation}
                            </Text>
                        </div>
                        <div
                            class="example-recall-actions"
                            data-testid="lesson-example-recall-actions"
                        >
                            // Self-assessment ON the answer side, after the
                            // translation was seen. Each choice advances
                            // (rating-free) — the same interaction shape as
                            // the rating buttons on every other card.
                            <Button
                                variant=ButtonVariant::Olive
                                on_click=Callback::new(move |_| on_advance.run(()))
                                test_id="lesson-example-understood"
                            >
                                {i18n
                                    .get_keys()
                                    .lesson()
                                    .example_recall_understood()
                                    .inner()
                                    .to_string()}
                            </Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                on_click=Callback::new(move |_| on_advance.run(()))
                                test_id="lesson-example-not-understood"
                            >
                                {i18n
                                    .get_keys()
                                    .lesson()
                                    .example_recall_not_understood()
                                    .inner()
                                    .to_string()}
                            </Button>
>>>>>>> origin/master
                        </div>
                    }
                    .into_any()
                }}
            </div>
<<<<<<< HEAD

            <Show when=move || !show_answer.get() && !load_failed.get() && example.get().is_some()>
                {move || {
                    view! {
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Filled)
                            on_click=Callback::new(move |_| on_show_answer.run(()))
                            test_id=Signal::derive(|| "lesson-example-reveal".to_string())
                        >
                            {t!(i18n, lesson.example_reveal_translation)}
                            <span class="kbd-hint">{t!(i18n, lesson.space_key)}</span>
                        </Button>
                    }
                }}
            </Show>
        </Card>

        // Binary self-assessment BELOW the card — the RatingButtons
        // etalon: «didn't understand [1]» (default) on the left,
        // «understood [2]» (olive) on the right. Each choice advances
        // rating-free immediately.
        {move || {
            if load_failed.get() {
                return view! {
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Filled)
                        on_click=Callback::new(move |_| on_advance.run(()))
                        test_id=Signal::derive(|| "lesson-example-next".to_string())
                        class=Signal::derive(|| "mt-4".to_string())
                    >
                        {t!(i18n, lesson.next)}
                    </Button>
                }
                .into_any();
            }
            if !show_answer.get() {
                return ().into_any();
            }
            view! {
                <div class="grid grid-cols-2 gap-3 mt-4" data-testid="lesson-example-recall-actions">
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Default)
                        on_click=Callback::new(move |_| on_advance.run(()))
                        test_id=Signal::derive(|| "lesson-example-not-understood".to_string())
                    >
                        {t!(i18n, lesson.example_recall_not_understood)}
                        <span class="kbd-hint">"[1]"</span>
                    </Button>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        on_click=Callback::new(move |_| on_advance.run(()))
                        test_id=Signal::derive(|| "lesson-example-understood".to_string())
                    >
                        {t!(i18n, lesson.example_recall_understood)}
                        <span class="kbd-hint">"[2]"</span>
                    </Button>
                </div>
            }
            .into_any()
        }}
=======
        </Card>
>>>>>>> origin/master
    }
}

/// The example card is locale-agnostic in rendering, but the translation
/// line must follow the app locale; the lesson context carries it and is
/// always present in the real lesson mount.
fn lesson_language() -> NativeLanguage {
    use_context::<super::LessonContext>()
        .map(|ctx| ctx.native_language.get_untracked())
        .unwrap_or(NativeLanguage::English)
}
