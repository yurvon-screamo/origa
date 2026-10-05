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
    on_advance: Callback<()>,
    known_kanji: HashSet<char>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let i18n = use_i18n();
    let word = match &card {
        Card::Vocabulary(vocab) => vocab.word().text().to_string(),
        _ => String::new(),
    };
    let word = StoredValue::new(word);
    let known = StoredValue::new(known_kanji);

    let example: RwSignal<Option<WordExample>> = RwSignal::new(None);
    spawn_local(async move {
        if let Ok(detail) = load_example_detail(sentence_id).await {
            example.set(Some(WordExample {
                detail,
                // offsets are word-specific and unknown in this view — the
                // highlight is re-located from the word surface
                start: -1,
                end: -1,
            }));
        }
    });

    let understood: RwSignal<Option<bool>> = RwSignal::new(None);

    view! {
        <div class="example-recall" data-testid="lesson-example-recall">
            <div class="example-recall-ja">
                {move || {
                    let Some(we) = example.get() else {
                        return ().into_any();
                    };
                    let word = word.get_value();
                    let (head, mid, tail) = highlight_parts(&we, &word);
                    let known = known.get_value();
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
                let Some(we) = example.get() else {
                    return ().into_any();
                };
                let translation = we
                    .detail
                    .translation(&native_language)
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
                                        .words()
                                        .detail_example_understood()
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
                                        .words()
                                        .detail_example_not_understood()
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
                // translation has been revealed).
                if understood.get().is_none() {
                    return ().into_any();
                }
                view! {
                    <Button
                        variant=ButtonVariant::Olive
                        on_click=Callback::new(move |_| on_advance.run(()))
                        test_id="lesson-example-next"
                    >
                        {crate::i18n::t!(i18n, lesson.next)}
                    </Button>
                }
                .into_any()
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
