//! Compact textbook example on a vocabulary card's answer side (#528).
//! Question sides never show it: the example would leak the translation.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::loaders::example_loader::{WordExample, load_word_examples};
use crate::pages::words::split_highlight;
use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[component]
pub fn WordExampleLine(
    word: String,
    known_kanji: HashSet<char>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let example: RwSignal<Option<WordExample>> = RwSignal::new(None);
    let word = StoredValue::new(word);
    let known = StoredValue::new(known_kanji);
    {
        spawn_local(async move {
            let loaded = load_word_examples(word.get_value().as_str()).await;
            if let Some(Ok(we)) = loaded.into_iter().next() {
                example.set(Some(we));
            }
        });
    }

    move || {
        let Some(we) = example.get() else {
            return ().into_any();
        };
        let WordExample { detail, start, end } = we;
        let word = word.get_value();
        // Stored offsets win; when absent (kana variant of a kanji word),
        // fall back to locating the surface form in the sentence.
        let (head, mid, tail) = if start >= 0 && end >= 0 {
            split_highlight(&detail.text, start, end)
        } else {
            match detail.text.find(&word) {
                Some(b) => {
                    let ci = detail.text[..b].chars().count();
                    split_highlight(&detail.text, ci as i32, (ci + word.chars().count()) as i32)
                },
                None => split_highlight(&detail.text, -1, -1),
            }
        };
        let translation = detail
            .translation(&native_language)
            .unwrap_or_default()
            .to_string();
        let known = known.get_value();
        view! {
            <div class="word-example-line" data-testid="lesson-word-example">
                <div class="word-example-line-ja">
                    <FuriganaText text=head.clone() known_kanji=known.clone()/>
                    <FuriganaText
                        text=mid.clone()
                        known_kanji=known.clone()
                        class=String::from("word-example-highlight")
                    />
                    <FuriganaText text=tail.clone() known_kanji=known/>
                </div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {translation}
                </Text>
            </div>
        }
        .into_any()
    }
}
