//! Example-recall view (#528): the question side shows a textbook sentence
//! using the word — no translation; the learner reveals the translation
//! first (the answer MUST be visible before the self-assessment, same
//! order as every other card), then rates understood / didn't understand;
//! the rating buttons advance the card. The self-assessment never touches
//! FSRS: advancing uses a rating-free lesson-state transition.

use leptos::prelude::*;
use leptos::task::spawn_local;

use super::LESSON_CARD_CLASS;
use crate::i18n::use_i18n;
use crate::loaders::example_loader::{WordExample, load_example_detail};
use crate::pages::words::split_highlight;
use crate::ui_components::{
    AudioButtons, Button, ButtonVariant, Card, FuriganaText, Text, TextSize, TypographyVariant,
};
use origa::domain::{Card, NativeLanguage};
use std::collections::HashSet;

#[component]
pub fn ExampleRecallCard(
    card: Card,
    sentence_id: u32,
    /// Char offsets of the word inside the sentence, carried by
    /// `LessonCardView::Example` from the CDN index (-1 = unlocated).
    start: i32,
    end: i32,
    on_advance: Callback<()>,
    known_kanji: Signal<HashSet<char>>,
    native_language: Signal<NativeLanguage>,
) -> impl IntoView {
    let i18n = use_i18n();
    let word = match &card {
        Card::Vocabulary(vocab) => vocab.word().text().to_string(),
        _ => String::new(),
    };
    let word = StoredValue::new(word);

    let example: RwSignal<Option<WordExample>> = RwSignal::new(None);
    // A failed chunk load (CDN 404, offline, corrupt file) must not turn
    // the card into a dead end: the fallback renders an explicit
    // «unavailable» notice with the advance button, so the lesson always
    // has a way forward.
    let load_failed: RwSignal<bool> = RwSignal::new(false);
    let tts_spoken = RwSignal::new(false);
    spawn_local(async move {
        match load_example_detail(sentence_id).await {
            Ok(detail) => example.set(Some(WordExample { detail, start, end })),
            Err(_) => load_failed.set(true),
        }
    });

    // Auto-speak the sentence once it renders — same as the question side
    // of every other lesson card. Respects the lesson mute toggle when the
    // lesson context is around (headless mounts have none).
    let lesson_ctx = use_context::<super::LessonContext>();
    Effect::new(move |_| {
        let Some(we) = example.get() else {
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
                        view! {
                            <AudioButtons
                                text=we.detail.text
                                audio_path=None
                                test_id=Signal::derive(|| "lesson-example-audio".to_string())
                            />
                        }
                    }}
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
                        }
                        .into_any();
                    }
                    view! {
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
                        </div>
                    }
                    .into_any()
                }}
            </div>
        </Card>
    }
}

fn highlight_parts(we: &WordExample, word: &str) -> (String, String, String) {
    let (start, end) = if we.start >= 0 && we.end >= 0 {
        (we.start, we.end)
    } else {
        match we.detail.text.find(word) {
            Some(b) => {
                let ci = we.detail.text[..b].chars().count();
                (ci as i32, (ci + word.chars().count()) as i32)
            },
            None => (-1, -1),
        }
    };
    split_highlight(&we.detail.text, start, end)
}
