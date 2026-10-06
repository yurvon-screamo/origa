//! Example-recall view (#528): the question side shows a textbook sentence
//! using the word — no translation; the learner self-assesses
//! understood / didn't understand, the translation is revealed, and the
//! card advances. The self-assessment never touches FSRS: advancing uses
//! a rating-free lesson-state transition.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::i18n::use_i18n;
use crate::loaders::example_loader::{WordExample, load_example_detail};
use crate::pages::words::split_highlight;
use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, Text, TextSize, TypographyVariant,
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
    spawn_local(async move {
        match load_example_detail(sentence_id).await {
            Ok(detail) => example.set(Some(WordExample { detail, start, end })),
            Err(_) => load_failed.set(true),
        }
    });

    let understood: RwSignal<Option<bool>> = RwSignal::new(None);

    let next_button = move || {
        view! {
            <Button
                variant=ButtonVariant::Olive
                on_click=Callback::new(move |_| on_advance.run(()))
                test_id="lesson-example-next"
            >
                {crate::i18n::t!(i18n, lesson.next)}
            </Button>
        }
    };

    view! {
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
                        {next_button()}
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

                match understood.get() {
                    None => {
                        view! {
                            <div
                                class="example-recall-actions"
                                data-testid="lesson-example-recall-actions"
                            >
                                <Button
                                    variant=ButtonVariant::Olive
                                    on_click=Callback::new(move |_| understood.set(Some(true)))
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
                                    on_click=Callback::new(move |_| understood.set(Some(false)))
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
                    },
                    Some(_) => view! {
                        <div
                            class="example-recall-translation"
                            data-testid="lesson-example-translation"
                        >
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {translation}
                            </Text>
                        </div>
                    }
                    .into_any(),
                }
            }}

            {move || {
                // Advance appears once the learner self-assessed (the
                // translation has been revealed). The self-assessment
                // value itself is intentional design (#528): a reflection
                // affordance with NO downstream effect — the owner
                // explicitly rejected tying it to FSRS or any gating.
                if understood.get().is_none() {
                    return ().into_any();
                }
                next_button().into_any()
            }}
        </div>
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
